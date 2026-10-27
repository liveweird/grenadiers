use grenadiers_core::{
    cpu::{
        Intent, MemoryKey, WorkBudget,
        forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast},
        policy::{ControlPlan, UtilityMemory},
        routing::{RouteLimits, RouteMode, RouteRequest, RouteStep, plan_route},
        survival::{ExecutionCertificate, SurvivalPlan},
    },
    game::{
        ActorInit, Arena, CellId, Direction, InputAction, Position, RulesConfig, SeatId, SeatInput,
        SpeedLevel, State, Terrain, Tick, World,
    },
    room::RoomSettings,
};
use std::collections::BTreeSet;
use tokio::time::Instant;

use super::tests::{
    open_fixture_state, publish_next_cpu_observation, running_driver, running_driver_with_fuse,
};

#[test]
fn driver_tick_does_not_cancel_an_ordinary_pending_worker_request() {
    let mut driver = running_driver(23);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let _ = publish_next_cpu_observation(&mut driver);
    assert!(driver.cpu.as_ref().unwrap().has_pending());
    assert!(!driver.cpu.as_ref().unwrap().has_committed_route());

    driver.tick(Instant::now()).unwrap();

    assert!(driver.cpu.as_ref().unwrap().has_pending());
    assert!(!driver.cpu.as_ref().unwrap().has_committed_route());
}

#[test]
fn committed_route_finishes_without_a_worker_result() {
    let mut driver = running_driver(23);
    driver
        .runtime
        .replace_state_for_test(open_fixture_state(driver.runtime.lifecycle().tick()));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let admission = driver.runtime.lifecycle().tick();
    let survive_until = Tick(admission.0 + 20);
    let route = plan_route(
        &basis,
        &forecast,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(admission.0 + 1),
                end: Tick(admission.0 + 2),
            },
            settled_stop_tick: basis.parts().tick,
            survive_until,
            mode: RouteMode::CertifiedOnly,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert!(matches!(
        route.steps()[0],
        grenadiers_core::cpu::routing::RouteStep::Wait { .. }
    ));
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis,
        driver.runtime.lifecycle().round_attempt(),
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent::STOP,
                hold_ticks: 1,
                next_memory: UtilityMemory::new(key),
            },
            lease: None,
            execution: Some(ExecutionCertificate::Route(Box::new(route))),
        },
    );

    driver.poll_cpu_result().unwrap();
    assert!(driver.cpu.as_ref().unwrap().has_committed_route());
    assert!(!driver.cpu.as_ref().unwrap().has_pending());

    while driver.runtime.lifecycle().tick() <= survive_until {
        driver.tick(Instant::now()).unwrap();
    }

    assert!(!driver.cpu.as_ref().unwrap().has_committed_route());
    assert!(driver.cpu_stationary);
}

#[test]
fn real_driver_executes_a_turning_multi_leg_route_through_its_predicted_explosion() {
    let (mut driver, survive_until, _) = admit_turning_route();

    let mut saw_expected_bomb_removal = false;
    let mut route_ended_at = None;
    while driver.runtime.lifecycle().tick() <= survive_until {
        driver.tick(Instant::now()).unwrap();
        let active = driver.cpu.as_ref().unwrap().has_committed_route();
        if active {
            assert!(!driver.cpu.as_ref().unwrap().has_pending());
        }
        if active
            && driver
                .last
                .cpu_observation
                .as_ref()
                .is_some_and(|observation| {
                    matches!(&observation.parts().bombs, grenadiers_core::cpu::BombKnowledge::Visible(bombs) if bombs.is_empty())
                })
        {
            saw_expected_bomb_removal = true;
        }
        if !active && route_ended_at.is_none() {
            route_ended_at = Some(driver.runtime.lifecycle().tick());
        }
    }
    assert!(
        saw_expected_bomb_removal,
        "route ended at {route_ended_at:?}, phase {:?}, last observation {:?}",
        driver.runtime.lifecycle().phase(),
        driver
            .last
            .cpu_observation
            .as_ref()
            .map(|observation| (&observation.parts().bombs, observation.parts().tick))
    );
    assert!(!driver.cpu.as_ref().unwrap().has_committed_route());
    assert!(driver.cpu_stationary);
}

#[test]
fn rejected_scheduled_stop_invalidates_route_without_rng_drift() {
    let (mut driver, _, first_stop) = admit_turning_route();
    let deadline = Tick(first_stop.0 - 1);
    while driver.runtime.lifecycle().tick() < deadline {
        driver.tick(Instant::now()).unwrap();
    }
    let draws = driver.cpu.as_ref().unwrap().rng_draws();
    let epoch = driver.runtime.lifecycle().round_attempt();
    for _ in 0..8 {
        driver
            .runtime
            .action(
                SeatId::Two,
                epoch,
                crate::game::rooms::match_runtime::MatchAction::Movement(Some(Direction::East)),
            )
            .unwrap();
    }

    driver.apply_cpu_route().unwrap();

    assert!(!driver.cpu.as_ref().unwrap().has_committed_route());
    assert_eq!(driver.cpu.as_ref().unwrap().rng_draws(), draws);
    assert_eq!(driver.cpu_stop_at, Some(deadline));
}

fn admit_turning_route() -> (super::super::Driver, Tick, Tick) {
    let mut driver = running_driver_with_fuse(23, 180);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(turning_bomb_state(tick));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let admission = driver.runtime.lifecycle().tick();
    let survive_until = Tick(admission.0 + 140);
    let route = plan_route(
        &basis,
        &forecast,
        RouteRequest {
            start_window: TickWindow {
                start: Tick(admission.0 + 1),
                end: Tick(admission.0 + 2),
            },
            settled_stop_tick: basis.parts().tick,
            survive_until,
            mode: RouteMode::AllowStationaryAssumptions,
            limits: RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let directions = route
        .steps()
        .iter()
        .filter_map(|step| match step {
            RouteStep::Move { direction, .. } => Some(*direction),
            RouteStep::Wait { .. } => None,
        })
        .collect::<Vec<_>>();
    assert!(
        directions.len() >= 2,
        "fixture must require multiple pulses"
    );
    let initial_heading = basis
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            grenadiers_core::cpu::ObservedActor::Visible(actor) if actor.seat == SeatId::Two => {
                Some(actor.heading)
            }
            _ => None,
        })
        .unwrap();
    assert_ne!(directions[0], initial_heading, "fixture must turn the CPU");
    let lane = route.lane(Tick(admission.0 + 1)).unwrap();
    let first_stop = lane.stop_update(0).unwrap();
    assert_eq!(lane.step_start(1), Some(Tick(first_stop.0 + 3)));
    let (movement, hold_ticks) = match route.steps()[0] {
        RouteStep::Move { direction, updates } => (
            grenadiers_core::cpu::MovementIntent::Direction(direction),
            updates,
        ),
        RouteStep::Wait { updates } => (grenadiers_core::cpu::MovementIntent::Stop, updates as u8),
    };
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis,
        driver.runtime.lifecycle().round_attempt(),
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement,
                    manual: None,
                },
                hold_ticks,
                next_memory: UtilityMemory::new(key),
            },
            lease: None,
            execution: Some(ExecutionCertificate::Route(Box::new(route))),
        },
    );
    driver.poll_cpu_result().unwrap();
    (driver, survive_until, first_stop)
}

fn turning_bomb_state(tick: Tick) -> State {
    use grenadiers_core::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};

    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || column == ARENA_WIDTH - 1 || row == 0 || row == ARENA_HEIGHT - 1 {
                terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(column)] =
                    Terrain::Wall;
            }
        }
    }
    let cpu_spawn = CellId::new(4, 7);
    let blocks = BTreeSet::from([CellId::new(4, 5)]);
    let arena = Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: grenadiers_core::game::EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::Two {
            cpu_spawn
        } else {
            CellId::new(5, 7)
        }),
        heading: if seat == SeatId::Two {
            Direction::West
        } else {
            Direction::East
        },
        speed: SpeedLevel::Four,
    });
    let start = Tick(tick.0 - 90);
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::new(SpeedLevel::Four, 64, 1, 180).unwrap()),
        World::new(arena),
        actors,
        start,
    )
    .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::PlaceBomb,
        }])
        .unwrap();
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Press(Direction::East),
        }])
        .unwrap();
    for _ in 0..60 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Clear,
        }])
        .unwrap();
    while state.tick() < tick {
        state.step(&[]).unwrap();
    }
    state
}
