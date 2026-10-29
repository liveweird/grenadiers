use crate::cpu::forecast::TickWindow;
use crate::cpu::{
    BombKnowledge, ObservedActor, ObservedBomb, ObservedBombTrigger, ObservedFlame,
    UnknownBombFacts, WorkBudget,
};
use crate::game::{
    BlastKind, CellId, Direction, EntityId, InputAction, SeatId, SeatInput, SpeedLevel, Tick,
};
use crate::room::RoomSettings;

use super::tests::{actual_state, forecast_for, observation, with_bomb};
use super::*;

fn request() -> RouteRequest {
    RouteRequest {
        start_window: TickWindow {
            start: Tick(1),
            end: Tick(8),
        },
        settled_stop_tick: Tick(0),
        survive_until: Tick(20),
        mode: RouteMode::CertifiedOnly,
        limits: RouteLimits::default(),
    }
}

#[test]
fn safe_stopped_route_is_explicit_wait_with_all_exact_start_lanes() {
    let observed = observation(Vec::new(), Vec::new());
    let forecast = forecast_for(&observed);
    let certificate = plan_route(
        &observed,
        &forecast,
        request(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(
        certificate,
        plan_route(
            &observed,
            &forecast,
            request(),
            &mut WorkBudget::new(65_536),
        )
        .unwrap()
    );

    assert_eq!(certificate.steps(), &[RouteStep::Wait { updates: 1 }]);
    for first in 1..=7 {
        let lane = certificate.lane(Tick(first)).unwrap();
        assert_eq!(lane.step_start(0), Some(Tick(first)));
        assert_eq!(lane.stop_update(0), Some(Tick(first)));
        assert_eq!(lane.ready_update(0), Some(Tick(first + 1)));
        assert!(lane.pose_at(Tick(0)).is_some());
        assert!(lane.pose_at(Tick(19)).is_some());
        assert!(lane.pose_at(Tick(20)).is_none());
    }
    assert!(certificate.lane(Tick(0)).is_none());
    assert!(certificate.lane(Tick(8)).is_none());
}

#[test]
fn legacy_stationary_route_work_is_pinned() {
    let observed = observation(Vec::new(), Vec::new());
    let forecast = forecast_for(&observed);
    let mut work = WorkBudget::new(65_536);
    plan_route(&observed, &forecast, request(), &mut work).unwrap();
    assert_eq!(work.spent(), 363);
}

#[test]
fn delayed_singleton_start_lane_is_certified_exactly() {
    let observed = observation(Vec::new(), Vec::new());
    let forecast = forecast_for(&observed);
    let mut delayed = request();
    delayed.start_window = TickWindow {
        start: Tick(7),
        end: Tick(8),
    };
    let certificate =
        plan_route(&observed, &forecast, delayed, &mut WorkBudget::new(65_536)).unwrap();
    assert!(certificate.lane(Tick(6)).is_none());
    assert!(certificate.lane(Tick(7)).is_some());
    assert!(certificate.lane(Tick(8)).is_none());
}

#[test]
fn route_limits_and_partial_work_fail_without_a_certificate() {
    let observed = observation(Vec::new(), Vec::new());
    let forecast = forecast_for(&observed);
    let mut invalid = request();
    invalid.limits.max_steps = MAX_ROUTE_STEPS + 1;
    assert_eq!(
        plan_route(&observed, &forecast, invalid, &mut WorkBudget::new(65_536)),
        Err(RouteFailure::InvalidRequest)
    );
    assert_eq!(
        plan_route(&observed, &forecast, request(), &mut WorkBudget::new(1)),
        Err(RouteFailure::WorkExhausted)
    );
}

#[test]
fn expansion_cap_never_returns_a_partial_escape() {
    let observed = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
    );
    let forecast = forecast_for(&observed);
    let mut route_request = request();
    route_request.survive_until = Tick(100);
    route_request.mode = RouteMode::AllowStationaryAssumptions;
    route_request.limits.max_expansions = 1;
    assert_eq!(
        plan_route(
            &observed,
            &forecast,
            route_request,
            &mut WorkBudget::new(65_536),
        ),
        Err(RouteFailure::Capacity)
    );
}

#[test]
fn multi_leg_escape_matches_actual_motion_and_stop_three_schedule() {
    let observed = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
    );
    let forecast = forecast_for(&observed);
    let mut route_request = request();
    route_request.survive_until = Tick(100);
    route_request.mode = RouteMode::AllowStationaryAssumptions;
    let mut work = WorkBudget::new(65_536);
    let certificate = plan_route(&observed, &forecast, route_request, &mut work).unwrap();
    assert_eq!(work.spent(), 4_355);
    assert!(work.spent() < 65_536);
    assert!(
        certificate
            .steps()
            .iter()
            .filter(|step| matches!(step, RouteStep::Move { .. }))
            .count()
            >= 2
    );
    if matches!(
        certificate.assurance(),
        RouteAssurance::ConditionalStationary { .. }
    ) {
        assert!(
            certificate
                .steps()
                .iter()
                .all(|step| { !matches!(step, RouteStep::Move { updates, .. } if *updates > 3) })
        );
    }

    let lane = certificate.lane(Tick(1)).unwrap();
    let ObservedActor::Visible(actor) = &observed.parts().actors[1] else {
        unreachable!()
    };
    let mut state = actual_state(
        actor.position,
        actor.heading,
        actor.speed,
        RoomSettings::DEFAULT,
    );
    for (index, step) in certificate.steps().iter().enumerate() {
        match *step {
            RouteStep::Move { direction, updates } => {
                for _ in 0..updates {
                    state
                        .step(&[SeatInput {
                            seat: SeatId::Two,
                            action: InputAction::Press(direction),
                        }])
                        .unwrap();
                    let actual = &state.actors()[SeatId::Two.index()];
                    assert_eq!(
                        lane.pose_at(state.tick()),
                        Some((actual.position(), actual.heading()))
                    );
                }
                state
                    .step(&[SeatInput {
                        seat: SeatId::Two,
                        action: InputAction::Clear,
                    }])
                    .unwrap();
                state.step(&[]).unwrap();
                state.step(&[]).unwrap();
                assert_eq!(lane.ready_update(index), Some(Tick(state.tick().0 + 1)));
            }
            RouteStep::Wait { updates } => {
                for _ in 0..updates {
                    state
                        .step(&[SeatInput {
                            seat: SeatId::Two,
                            action: InputAction::Clear,
                        }])
                        .unwrap();
                }
            }
        }
    }
}

#[test]
fn bounded_search_finds_required_alignment_then_turn() {
    let mut parts = with_bomb(
        observation(Vec::new(), Vec::new()),
        ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
    )
    .into_parts();
    parts.arena.blocks.push(CellId::new(1, 1));
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position.x += 400;
    actor.heading = Direction::East;
    actor.speed = SpeedLevel::Four;
    let observed = crate::cpu::Observation::new(parts).unwrap();
    let forecast = forecast_for(&observed);
    let mut route_request = request();
    route_request.survive_until = Tick(100);
    route_request.mode = RouteMode::AllowStationaryAssumptions;
    let certificate = plan_route(
        &observed,
        &forecast,
        route_request,
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let directions = certificate
        .steps()
        .iter()
        .filter_map(|step| match step {
            RouteStep::Move { direction, .. } => Some(*direction),
            RouteStep::Wait { .. } => None,
        })
        .collect::<Vec<_>>();
    assert!(directions.windows(2).any(|pair| {
        matches!(pair[0], Direction::East | Direction::West)
            && matches!(pair[1], Direction::North | Direction::South)
    }));
}

#[test]
fn route_waits_for_a_flame_gap_then_moves_to_refuge() {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.bombs = BombKnowledge::Visible(vec![ObservedBomb {
        id: EntityId(90),
        placed_at: crate::game::Tick(0),
        cell: CellId::new(2, 4),
        owner: EntityId(1),
        range: 3,
        trigger: ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
        blast: BlastKind::Standard,
        unknown: UnknownBombFacts::ALL,
    }]);
    parts.arena.blocks.push(CellId::new(1, 1));
    parts.flames = vec![
        ObservedFlame {
            cell: CellId::new(3, 1),
            remaining_ticks: 12,
        },
        ObservedFlame {
            cell: CellId::new(2, 2),
            remaining_ticks: 12,
        },
    ];
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position.x -= 128;
    actor.position.y -= 128;
    actor.speed = SpeedLevel::Four;
    actor.heading = Direction::East;
    let observed = crate::cpu::Observation::new(parts).unwrap();
    let forecast = forecast_for(&observed);
    let mut route_request = request();
    route_request.survive_until = Tick(100);
    route_request.mode = RouteMode::AllowStationaryAssumptions;
    route_request.limits.max_steps = 5;
    let certificate = plan_route(
        &observed,
        &forecast,
        route_request,
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert!(matches!(
        certificate.steps().first(),
        Some(RouteStep::Wait { updates }) if *updates > 1
    ));
    assert!(
        certificate.steps()[1..]
            .iter()
            .any(|step| matches!(step, RouteStep::Move { .. }))
    );
}
