use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::tests::{key, observation};
use crate::cpu::routing::{
    RouteCertificate, RouteLimits, RouteMode, RouteRequest, RouteStep, plan_route,
};
use crate::cpu::{MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::{CellId, Direction, Position, Tick};

use super::tests::later;
use super::{CommittedRoute, RouteStatus, WORK_ALLOWANCE};

fn multi_leg_route() -> (Observation, RouteCertificate) {
    let mut parts = observation(CellId::new(2, 1)).into_parts();
    parts.tick = Tick(0);
    parts.rules.running_updates = 0;
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.heading = Direction::East;
    parts.actors[0] = ObservedActor::Visible(crate::cpu::VisibleActor {
        id: crate::game::EntityId(1),
        seat: crate::game::SeatId::One,
        position: Position::center(CellId::new(13, 11)),
        heading: Direction::East,
        speed: crate::game::SpeedLevel::Base,
        alive: true,
        bomb_capacity: 1,
        bomb_range: 2,
        live_bomb_count: 0,
        equipment: None,
        weapon: None,
        effects: Vec::new(),
        damage_grace_remaining_ticks: 0,
        voluntary_actions_frozen: false,
        last_teleport_tick: None,
        unknown: crate::cpu::UnknownActorFacts::ALL,
    });
    parts.bombs = crate::cpu::BombKnowledge::Visible(vec![crate::cpu::ObservedBomb {
        id: crate::game::EntityId(90),
        placed_at: crate::game::Tick(0),
        cell: CellId::new(5, 1),
        owner: crate::game::EntityId(1),
        range: 3,
        trigger: crate::cpu::ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
        blast: crate::game::BlastKind::Standard,
        unknown: crate::cpu::UnknownBombFacts::ALL,
    }]);
    let basis = Observation::new(parts).unwrap();
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let route = plan_route(
        &basis,
        &hazards,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(1),
                end: Tick(8),
            },
            settled_stop_tick: Tick(0),
            survive_until: Tick(100),
            mode: RouteMode::AllowStationaryAssumptions,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    (basis, route)
}

pub(super) fn cleared_wait_route() -> (Observation, RouteCertificate, Observation) {
    let mut parts = observation(CellId::new(2, 1)).into_parts();
    parts.bombs = crate::cpu::BombKnowledge::Visible(vec![crate::cpu::ObservedBomb {
        id: crate::game::EntityId(91),
        placed_at: Tick(10),
        cell: CellId::new(10, 10),
        owner: crate::game::EntityId(1),
        range: 1,
        trigger: crate::cpu::ObservedBombTrigger::Timed {
            remaining_ticks: 100,
            paused: false,
        },
        blast: crate::game::BlastKind::Standard,
        unknown: crate::cpu::UnknownBombFacts::ALL,
    }]);
    let basis = Observation::new(parts).unwrap();
    let hazards = forecast(
        &basis,
        &ObservedHazardMemory::new(key()),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    let route = plan_route(
        &basis,
        &hazards,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(11),
                end: Tick(18),
            },
            settled_stop_tick: Tick(10),
            survive_until: hazards.horizon_end(),
            mode: RouteMode::CertifiedOnly,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(
        route
            .steps()
            .iter()
            .all(|step| matches!(step, RouteStep::Wait { .. }))
    );
    let mut clear = basis.clone().into_parts();
    clear.tick = Tick(150);
    clear.rules.running_updates += 140;
    clear.rules.remaining_ticks -= 140;
    clear.bombs = crate::cpu::BombKnowledge::Visible(Vec::new());
    (basis, route, Observation::new(clear).unwrap())
}

#[test]
fn stopped_wait_completes_only_after_its_visible_hazard_fully_clears() {
    let (basis, route, clear) = cleared_wait_route();
    let admit = || {
        CommittedRoute::admit(
            route.clone(),
            basis.clone(),
            &basis,
            Tick(10),
            MovementIntent::Stop,
        )
        .unwrap()
    };
    assert_eq!(
        admit().observe_with_stop(&clear, Some(Tick(11))),
        RouteStatus::Completed
    );
    assert_eq!(admit().observe_with_stop(&clear, None), RouteStatus::Wait);

    let mut wrong_pose = clear.clone().into_parts();
    let ObservedActor::Visible(actor) = &mut wrong_pose.actors[1] else {
        unreachable!()
    };
    actor.position.x += 1;
    assert_eq!(
        admit().observe_with_stop(&Observation::new(wrong_pose).unwrap(), Some(Tick(11))),
        RouteStatus::Invalidated
    );
    let mut unknown = clear.clone().into_parts();
    unknown.bombs = crate::cpu::BombKnowledge::Unavailable;
    assert_eq!(
        admit().observe_with_stop(&Observation::new(unknown).unwrap(), Some(Tick(11))),
        RouteStatus::Invalidated
    );
    let mut flame = clear.clone().into_parts();
    flame.flames.push(crate::cpu::ObservedFlame {
        cell: CellId::new(10, 10),
        remaining_ticks: 1,
    });
    assert_eq!(
        admit().observe_with_stop(&Observation::new(flame).unwrap(), Some(Tick(11))),
        RouteStatus::Invalidated
    );
    let mut oil = clear.into_parts();
    oil.oil.push(crate::cpu::ObservedOil {
        cell: CellId::new(10, 10),
        state: crate::game::OilBarrelState::Burning,
    });
    assert_eq!(
        admit().observe_with_stop(&Observation::new(oil).unwrap(), Some(Tick(11))),
        RouteStatus::Invalidated
    );
    assert_eq!(
        admit().observe_with_stop(&basis, Some(Tick(11))),
        RouteStatus::Invalidated
    );
}

#[test]
fn every_start_lane_and_stop_deadline_are_exact() {
    let (basis, route) = multi_leg_route();
    assert!(
        route
            .steps()
            .iter()
            .filter(|step| matches!(step, RouteStep::Move { .. }))
            .count()
            >= 2
    );
    let first_movement = first_movement(&route);
    for admission in 0..=6 {
        assert!(
            CommittedRoute::admit(
                route.clone(),
                basis.clone(),
                &basis,
                Tick(admission),
                first_movement
            )
            .is_ok()
        );
    }
    let lane = route.lane(Tick(1)).unwrap().clone();
    let first_stop = lane.stop_update(0).unwrap();
    let missed = CommittedRoute::admit(
        route.clone(),
        basis.clone(),
        &basis,
        Tick(0),
        first_movement,
    )
    .unwrap();
    assert_eq!(
        missed.poll_command(first_stop, None),
        RouteStatus::Invalidated
    );
    let mut committed =
        CommittedRoute::admit(route, basis.clone(), &basis, Tick(0), first_movement).unwrap();
    let RouteStatus::Submit(stop) = committed.poll_command(Tick(first_stop.0 - 1), None) else {
        panic!("expected Stop")
    };
    assert_eq!(stop.movement(), MovementIntent::Stop);
    assert!(committed.commit_command(stop));
    assert_eq!(
        committed.poll_command(Tick(lane.ready_update(0).unwrap().0 - 1), Some(first_stop)),
        RouteStatus::Invalidated
    );
}

#[test]
fn exact_stop_confirmation_allows_the_second_leg_without_a_worker() {
    let (basis, route) = multi_leg_route();
    let lane = route.lane(Tick(1)).unwrap().clone();
    let first_movement = first_movement(&route);
    let mut committed =
        CommittedRoute::admit(route, basis.clone(), &basis, Tick(0), first_movement).unwrap();
    let stop_update = lane.stop_update(0).unwrap();
    let RouteStatus::Submit(stop) = committed.poll_command(Tick(stop_update.0 - 1), None) else {
        panic!("expected Stop")
    };
    assert!(committed.commit_command(stop));
    let (position, heading) = lane.pose_at(stop_update).unwrap();
    assert_eq!(
        committed.observe(&later(&basis, stop_update.0, position, heading)),
        RouteStatus::Wait
    );
    let RouteStatus::Submit(next) =
        committed.poll_command(Tick(lane.ready_update(0).unwrap().0 - 1), Some(stop_update))
    else {
        panic!("expected second leg")
    };
    assert_ne!(next.movement(), MovementIntent::Stop);
    assert!(committed.commit_command(next));
}

#[test]
fn route_rejects_bomb_range_change_at_admission_and_midroute() {
    let (basis, route) = multi_leg_route();
    let movement = first_movement(&route);
    let mut changed = basis.clone().into_parts();
    let ObservedActor::Visible(cpu) = &mut changed.actors[1] else {
        unreachable!()
    };
    cpu.bomb_range += 1;
    let changed = Observation::new(changed).unwrap();
    assert!(
        CommittedRoute::admit(route.clone(), basis.clone(), &changed, Tick(0), movement).is_err()
    );

    let mut committed =
        CommittedRoute::admit(route.clone(), basis.clone(), &basis, Tick(0), movement).unwrap();
    let (position, heading) = route.lane(Tick(1)).unwrap().pose_at(Tick(1)).unwrap();
    let mut changed = later(&basis, 1, position, heading).into_parts();
    let ObservedActor::Visible(cpu) = &mut changed.actors[1] else {
        unreachable!()
    };
    cpu.bomb_range += 1;
    assert_eq!(
        committed.observe(&Observation::new(changed).unwrap()),
        RouteStatus::Invalidated
    );
}

#[test]
fn leading_wait_requires_its_post_stop_snapshot_before_moving() {
    let mut parts = crate::cpu::routing::tests::observation(Vec::new(), Vec::new()).into_parts();
    parts.bombs = crate::cpu::BombKnowledge::Visible(vec![crate::cpu::ObservedBomb {
        id: crate::game::EntityId(90),
        placed_at: crate::game::Tick(0),
        cell: CellId::new(2, 4),
        owner: crate::game::EntityId(1),
        range: 3,
        trigger: crate::cpu::ObservedBombTrigger::Timed {
            remaining_ticks: 60,
            paused: false,
        },
        blast: crate::game::BlastKind::Standard,
        unknown: crate::cpu::UnknownBombFacts::ALL,
    }]);
    parts.arena.blocks.push(CellId::new(1, 1));
    parts.flames = vec![
        crate::cpu::ObservedFlame {
            cell: CellId::new(3, 1),
            remaining_ticks: 12,
        },
        crate::cpu::ObservedFlame {
            cell: CellId::new(2, 2),
            remaining_ticks: 12,
        },
    ];
    let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
        unreachable!()
    };
    cpu.speed = crate::game::SpeedLevel::Four;
    cpu.position.x -= 128;
    cpu.position.y -= 128;
    let basis = Observation::new(parts).unwrap();
    let hazards = crate::cpu::routing::tests::forecast_for(&basis);
    let limits = RouteLimits {
        max_steps: 5,
        ..RouteLimits::default()
    };
    let route = plan_route(
        &basis,
        &hazards,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(1),
                end: Tick(8),
            },
            settled_stop_tick: Tick(0),
            survive_until: Tick(100),
            mode: RouteMode::AllowStationaryAssumptions,
            limits,
        },
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .unwrap();
    assert!(
        matches!(route.steps()[0], RouteStep::Wait { .. }),
        "expected leading Wait: {:?}",
        route.steps()
    );
    let lane = route.lane(Tick(1)).unwrap().clone();
    let next_move = route
        .steps()
        .iter()
        .position(|step| matches!(step, RouteStep::Move { .. }))
        .unwrap_or_else(|| panic!("expected Move after Wait: {:?}", route.steps()));
    let move_admission = Tick(lane.step_start(next_move).unwrap().0 - 1);
    let mut missing = CommittedRoute::admit(
        route.clone(),
        basis.clone(),
        &basis,
        Tick(0),
        MovementIntent::Stop,
    )
    .unwrap();
    assert_eq!(
        missing.poll_command(move_admission, Some(Tick(1))),
        RouteStatus::Invalidated
    );

    let stopped = later(
        &basis,
        1,
        lane.pose_at(Tick(1)).unwrap().0,
        lane.pose_at(Tick(1)).unwrap().1,
    );
    assert_eq!(missing.observe(&stopped), RouteStatus::Wait);
    assert!(matches!(
        missing.poll_command(move_admission, Some(Tick(1))),
        RouteStatus::Submit(_)
    ));
}

fn first_movement(route: &RouteCertificate) -> MovementIntent {
    match route.steps()[0] {
        RouteStep::Move { direction, .. } => MovementIntent::Direction(direction),
        RouteStep::Wait { .. } => MovementIntent::Stop,
    }
}
