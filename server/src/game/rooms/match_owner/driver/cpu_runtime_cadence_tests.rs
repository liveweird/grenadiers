use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{Observation, ObservedActor, ObservedFlame},
    game::{
        ActorInit, Arena, CellId, Direction, EntityId, PickupKind, Position, RulesConfig, SeatId,
        SpeedLevel, State, Terrain, World,
    },
    room::{CpuProfile, RoomSettings},
};
use tokio::time::Instant;

use super::tests::{publish_next_cpu_observation, running_driver_and_planner_with_profile};

#[test]
fn real_easy_queue_progresses_then_new_danger_bypasses_the_bounded_pause() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 1, 60).unwrap();
    let (mut driver, planner) =
        running_driver_and_planner_with_profile(23, settings, CpuProfile::CURRENT_EASY);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(clear_goal_fixture(tick));
    let initial = driver.runtime.current_notice().unwrap();
    let origin = cpu_position(initial.cpu_observation.as_ref().unwrap());

    driver.consider_cpu_observation(&initial);
    publish_next_cpu_observation(&mut driver);
    assert!(driver.cpu.as_ref().unwrap().has_pending());
    let opening = planner.pump_for_test(usize::MAX);
    assert_eq!(opening.delivered, 1);
    assert_ne!(
        opening.planned_intents[0].movement,
        grenadiers_core::cpu::MovementIntent::Stop,
        "{opening:?}"
    );
    assert_eq!(opening.planned_intents[0].manual, None, "{opening:?}");
    driver.poll_cpu_result().unwrap();
    assert!(!driver.cpu_stationary, "the first decision stays immediate");

    let mut moved = false;
    for _ in 0..40 {
        driver.tick(Instant::now()).unwrap();
        moved |= driver
            .last
            .cpu_observation
            .as_ref()
            .is_some_and(|observation| cpu_position(observation) != origin);
        if driver.cpu_stationary && driver.cpu_stop_at.is_none() {
            break;
        }
    }
    assert!(moved, "the queued Easy plan must make physical progress");
    assert!(driver.cpu_stationary);
    assert!(!driver.cpu.as_ref().unwrap().has_pending());

    for _ in 0..3 {
        driver.tick(Instant::now()).unwrap();
        assert!(
            !driver.cpu.as_ref().unwrap().has_pending(),
            "safe repeated publications must retain the pause"
        );
    }

    let mut hazard = (0..3)
        .find_map(|_| driver.runtime.settle_boundary([None; 2]).unwrap())
        .expect("running round publishes the next observation");
    let mut parts = hazard.cpu_observation.take().unwrap().into_parts();
    parts.flames.push(ObservedFlame {
        cell: CellId::new(6, 6),
        remaining_ticks: 2,
    });
    hazard.cpu_observation = Some(Observation::new(parts).unwrap());
    driver.consider_cpu_observation(&hazard);

    assert!(
        driver.cpu.as_ref().unwrap().has_pending(),
        "new danger must bypass the remaining artificial pause"
    );
    assert_eq!(planner.pump_for_test(usize::MAX).delivered, 1);
    driver.poll_cpu_result().unwrap();
}

fn cpu_position(observation: &Observation) -> grenadiers_core::game::Position {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == SeatId::Two => Some(actor.position),
            _ => None,
        })
        .unwrap()
}

fn clear_goal_fixture(tick: grenadiers_core::game::Tick) -> State {
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
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(8, 7), PickupKind::BlastRange)]),
    )
    .unwrap();
    let actors = [SeatId::One, SeatId::Two].map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::Two {
            CellId::new(7, 7)
        } else {
            CellId::new(1, 1)
        }),
        heading: Direction::West,
        speed: SpeedLevel::Four,
    });
    State::new_at_tick(
        RulesConfig::expanded_v5(RoomSettings::new(SpeedLevel::Four, 64, 1, 60).unwrap()),
        world,
        actors,
        tick,
    )
    .unwrap()
}
