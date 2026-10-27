use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{MovementIntent, Observation, ObservedActor},
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, Equipment, PickupKind,
        Position, RulesConfig, SeatId, SpeedLevel, State, Terrain, Tick, World,
    },
    room::RoomSettings,
};
use tokio::time::Instant;

use super::super::Driver;
use super::tests::{publish_next_cpu_observation, running_driver_and_planner_with_settings};
use crate::game::rooms::cpu::CpuPlanner;

const CPU_CELL: CellId = CellId::new(7, 7);
const DIRECT_CELL: CellId = CellId::new(8, 7);
const GOAL_CELL: CellId = CellId::new(9, 7);

#[test]
fn queued_driver_deliberately_collects_reachable_punch_and_bracelet_when_empty_or_holding_kick() {
    for (pickup, equipment) in [
        (PickupKind::Punch, Equipment::Punch),
        (PickupKind::Bracelet, Equipment::Bracelet),
    ] {
        for (held, initial_equipment) in [
            (None, None),
            (Some(PickupKind::Kick), Some(Equipment::Kick)),
        ] {
            let (mut driver, planner) = prepared_driver(held, [(DIRECT_CELL, pickup)]);
            let basis = queued_basis(&mut driver);
            assert_eq!(cpu_actor(&basis).equipment, initial_equipment);
            assert_eq!(cpu_actor(&basis).position, Position::center(CPU_CELL));

            let report = planner.pump_for_test(1);
            assert_eq!((report.dequeued, report.delivered), (1, 1));
            assert_eq!(report.planned_intents.len(), 1);
            assert_eq!(
                report.planned_intents[0].movement,
                MovementIntent::Direction(Direction::East),
                "CPU with {initial_equipment:?} must deliberately seek {pickup:?}: {report:?}"
            );
            assert_eq!(report.planned_intents[0].manual, None);
            driver.poll_cpu_result().unwrap();

            run_until(&mut driver, &planner, 24, |state| {
                state.actors()[SeatId::Two.index()].inventory().equipment() == Some(equipment)
            });
            let state = &driver.runtime.sim().unwrap().state;
            assert_eq!(
                state.actors()[SeatId::Two.index()].inventory().equipment(),
                Some(equipment),
                "accepted ordinary movement must physically collect {pickup:?}"
            );
            assert!(!state.world().exposed_pickups().contains_key(&DIRECT_CELL));
        }
    }
}

#[test]
fn queued_driver_crosses_duplicate_punch_and_bracelet_without_replacing_them() {
    for (held_pickup, held_equipment) in [
        (PickupKind::Punch, Equipment::Punch),
        (PickupKind::Bracelet, Equipment::Bracelet),
    ] {
        let (mut driver, planner) = prepared_driver(
            Some(held_pickup),
            [
                (DIRECT_CELL, held_pickup),
                (GOAL_CELL, PickupKind::BombCapacity),
            ],
        );
        let basis = queued_basis(&mut driver);
        assert_eq!(cpu_actor(&basis).equipment, Some(held_equipment));

        let report = planner.pump_for_test(1);
        assert_eq!((report.dequeued, report.delivered), (1, 1));
        assert_eq!(
            report.planned_intents[0].movement,
            MovementIntent::Direction(Direction::East),
            "duplicate {held_pickup:?} must remain transitable: {report:?}"
        );
        driver.poll_cpu_result().unwrap();

        run_until(&mut driver, &planner, 60, |state| {
            state.actors()[SeatId::Two.index()].bomb_capacity() > 1
        });
        let actor = &driver.runtime.sim().unwrap().state.actors()[SeatId::Two.index()];
        assert_eq!(actor.bomb_capacity(), 2);
        assert_eq!(actor.inventory().equipment(), Some(held_equipment));
    }
}

#[test]
fn queued_driver_detours_around_equipment_that_would_replace_a_retained_item() {
    for (held_pickup, held_equipment, unwanted) in [
        (PickupKind::Punch, Equipment::Punch, PickupKind::Kick),
        (PickupKind::Punch, Equipment::Punch, PickupKind::Bracelet),
        (PickupKind::Bracelet, Equipment::Bracelet, PickupKind::Kick),
        (PickupKind::Bracelet, Equipment::Bracelet, PickupKind::Punch),
        (PickupKind::Heart, Equipment::Heart, PickupKind::Punch),
        (PickupKind::WallPass, Equipment::WallPass, PickupKind::Punch),
    ] {
        let (mut driver, planner) = prepared_driver(
            Some(held_pickup),
            [
                (DIRECT_CELL, unwanted),
                (GOAL_CELL, PickupKind::BombCapacity),
            ],
        );
        let basis = queued_basis(&mut driver);
        assert_eq!(cpu_actor(&basis).equipment, Some(held_equipment));

        let report = planner.pump_for_test(1);
        assert_eq!((report.dequeued, report.delivered), (1, 1));
        assert!(
            matches!(
                report.planned_intents[0].movement,
                MovementIntent::Direction(Direction::North | Direction::South)
            ),
            "held {held_equipment:?} must detour around {unwanted:?}: {report:?}"
        );
        driver.poll_cpu_result().unwrap();

        run_until(&mut driver, &planner, 84, |state| {
            state.actors()[SeatId::Two.index()].bomb_capacity() > 1
        });
        let state = &driver.runtime.sim().unwrap().state;
        let actor = &state.actors()[SeatId::Two.index()];
        assert_eq!(actor.bomb_capacity(), 2);
        assert_eq!(actor.inventory().equipment(), Some(held_equipment));
        assert!(state.world().exposed_pickups().contains_key(&DIRECT_CELL));
    }
}

fn queued_basis(driver: &mut Driver) -> Observation {
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    publish_next_cpu_observation(driver)
}

fn run_until(
    driver: &mut Driver,
    planner: &CpuPlanner,
    limit: usize,
    done: impl Fn(&State) -> bool,
) {
    for _ in 0..limit {
        if done(&driver.runtime.sim().unwrap().state) {
            return;
        }
        planner.pump_for_test(1);
        driver.poll_cpu_result().unwrap();
        driver.tick(Instant::now()).unwrap();
    }
}

fn cpu_actor(observation: &Observation) -> &grenadiers_core::cpu::VisibleActor {
    match &observation.parts().actors[SeatId::Two.index()] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => panic!("CPU observes itself"),
    }
}

fn prepared_driver(
    held: Option<PickupKind>,
    pickups: impl IntoIterator<Item = (CellId, PickupKind)>,
) -> (Driver, CpuPlanner) {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 1, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::ALL);
    let (mut driver, planner) = running_driver_and_planner_with_settings(23, settings);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(equipment_fixture(tick, settings, held, pickups));
    (driver, planner)
}

fn equipment_fixture(
    tick: Tick,
    settings: RoomSettings,
    held: Option<PickupKind>,
    pickups: impl IntoIterator<Item = (CellId, PickupKind)>,
) -> State {
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
    let mut exposed: BTreeMap<_, _> = pickups.into_iter().collect();
    if let Some(kind) = held {
        assert!(exposed.insert(CPU_CELL, kind).is_none());
    }
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::Two {
            CPU_CELL
        } else {
            CellId::new(1, 1)
        }),
        heading: Direction::East,
        speed: SpeedLevel::Four,
    });
    let initial_tick = held.map_or(tick, |_| Tick(tick.0 - 1));
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(settings),
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), exposed).unwrap(),
        actors,
        initial_tick,
    )
    .unwrap();
    if held.is_some() {
        state.step(&[]).unwrap();
        assert_eq!(state.tick(), tick);
    }
    state
}
