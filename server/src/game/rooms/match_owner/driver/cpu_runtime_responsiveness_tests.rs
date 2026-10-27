use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{
        BombKnowledge, CpuRng, MemoryKey, MovementIntent, Observation, ObservedActor, WorkBudget,
        forecast::ObservedHazardMemory,
        policy::UtilityMemory,
        routing::RouteStep,
        survival::{ExecutionCertificate, SurvivalPlan, WORK_ALLOWANCE, plan},
    },
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction, PickupKind,
        Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, Terrain, Tick, World,
    },
    room::RoomSettings,
};
use tokio::time::Instant;

use super::super::Driver;
use super::tests::{publish_next_cpu_observation, running_driver_with_settings};

#[test]
fn finite_flame_wait_releases_the_driver_to_collect_a_pickup() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 1, 60)
        .unwrap()
        .with_enabled_items(EnabledItems::INVISIBILITY_CLOAK);
    let mut driver = running_driver_with_settings(23, settings);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(pre_detonation_fixture(tick, settings));

    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    assert!(matches!(&basis.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(matches!(
        basis.parts().actors[SeatId::One.index()],
        ObservedActor::Hidden { alive: true, .. }
    ));
    assert_eq!(
        basis
            .parts()
            .flames
            .iter()
            .find(|flame| flame.cell == CellId::new(8, 7))
            .map(|flame| flame.remaining_ticks),
        Some(30)
    );

    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    let mut memory = UtilityMemory::new(key);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&basis).unwrap();
    let mut rng = CpuRng::from_seed(key, [91; 32]);
    let (waiting, initial_work) = planned(&basis, memory, &history, &mut rng);
    memory = waiting.control.next_memory;
    assert_eq!(waiting.control.intent.movement, MovementIntent::Stop);
    let route = match waiting.execution.as_ref() {
        Some(ExecutionCertificate::Route(route)) => route,
        execution => panic!("finite flame must produce a committed route, got {execution:?}"),
    };
    assert!(matches!(
        route.steps().first(),
        Some(RouteStep::Wait { .. })
    ));
    assert_eq!(route.survive_until(), Tick(basis.parts().tick.0 + 30));
    assert!(initial_work > 0 && initial_work <= WORK_ALLOWANCE);
    let basis_tick = basis.parts().tick;
    let expiry = route.survive_until();

    deliver(&mut driver, basis, waiting);
    assert!(driver.cpu.as_ref().unwrap().has_committed_route());
    assert!(driver.cpu_stationary);

    let route_ended_at = loop {
        driver.tick(Instant::now()).unwrap();
        if !driver.cpu.as_ref().unwrap().has_committed_route() {
            break driver.runtime.lifecycle().tick();
        }
        assert!(
            driver.cpu_stationary,
            "Wait route issued unexpected movement"
        );
        assert!(driver.runtime.lifecycle().tick() <= expiry);
    };
    assert_eq!(route_ended_at, expiry);
    let fresh = driver.last.cpu_observation.clone().unwrap();
    assert!(fresh.parts().flames.is_empty());
    history.observe(&fresh).unwrap();
    let (goal, fresh_work) = planned(&fresh, memory, &history, &mut rng);
    memory = goal.control.next_memory;
    assert!(matches!(
        goal.control.intent.movement,
        MovementIntent::Direction(_)
    ));
    assert!(fresh_work > 0 && fresh_work <= WORK_ALLOWANCE);
    deliver(&mut driver, fresh, goal);
    let first_goal_admission = driver.runtime.lifecycle().tick();
    assert!(
        !driver.cpu_stationary,
        "fresh goal movement was not admitted"
    );
    assert_eq!(
        first_goal_admission, expiry,
        "Driver stayed stopped after expiry"
    );

    let pickup_tick = loop {
        driver.tick(Instant::now()).unwrap();
        let observation = driver.last.cpu_observation.clone().unwrap();
        if cpu_actor(&observation).bomb_capacity == 2 {
            break observation.parts().tick;
        }
        assert!(observation.parts().tick.0 <= expiry.0 + 80);
        if driver.cpu_stationary && driver.cpu.as_ref().unwrap().has_pending() {
            history.observe(&observation).unwrap();
            let (next, _) = planned(&observation, memory, &history, &mut rng);
            memory = next.control.next_memory;
            assert!(matches!(
                next.control.intent.movement,
                MovementIntent::Direction(_)
            ));
            deliver(&mut driver, observation, next);
            assert!(!driver.cpu_stationary);
        }
    };
    assert_eq!(pickup_tick, Tick(expiry.0 + 21));
    eprintln!(
        "finite_driver_metric basis={} end={} span={} routeEnded={} firstMoveAdmitted={} pickup={} stoppedAfterExpiry={} initialWork={} freshWork={}",
        basis_tick.0,
        expiry.0,
        expiry.0 - basis_tick.0,
        route_ended_at.0,
        first_goal_admission.0,
        pickup_tick.0,
        first_goal_admission.0.saturating_sub(expiry.0),
        initial_work,
        fresh_work
    );
    assert!(
        driver
            .last
            .cpu_observation
            .as_ref()
            .unwrap()
            .parts()
            .pickups
            .iter()
            .all(|pickup| pickup.cell != CellId::new(9, 7))
    );
}

fn planned(
    observation: &Observation,
    memory: UtilityMemory,
    history: &ObservedHazardMemory,
    rng: &mut CpuRng,
) -> (SurvivalPlan, u32) {
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let plan = plan(
        observation,
        &memory,
        history,
        observation.parts().tick,
        rng,
        &mut work,
    )
    .unwrap();
    (plan, work.spent())
}

fn deliver(driver: &mut Driver, basis: Observation, plan: SurvivalPlan) {
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis,
        driver.runtime.lifecycle().round_attempt(),
        plan,
    );
    driver.poll_cpu_result().unwrap();
}

fn cpu_actor(observation: &Observation) -> &grenadiers_core::cpu::VisibleActor {
    match &observation.parts().actors[SeatId::Two.index()] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => panic!("CPU observes itself"),
    }
}

fn pre_detonation_fixture(tick: Tick, settings: RoomSettings) -> State {
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
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::Two {
            CellId::new(7, 7)
        } else {
            CellId::new(8, 6)
        }),
        heading: Direction::East,
        speed: SpeedLevel::Four,
    });
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (CellId::new(8, 6), PickupKind::InvisibilityCloak),
            (CellId::new(9, 7), PickupKind::BombCapacity),
        ]),
    )
    .unwrap();
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(settings),
        world,
        actors,
        Tick(tick.0 - 58),
    )
    .unwrap();
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::PlaceBomb,
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::East),
            },
        ])
        .unwrap();
    for _ in 0..23 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::East),
            },
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Press(Direction::North),
            },
        ])
        .unwrap();
    for _ in 0..23 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::One,
            action: InputAction::Release(Direction::North),
        }])
        .unwrap();
    for _ in 0..9 {
        state.step(&[]).unwrap();
    }
    assert_eq!(state.tick(), tick);
    assert_eq!(state.world().bombs().len(), 1);
    assert!(state.world().hazards().is_empty());
    state
}
