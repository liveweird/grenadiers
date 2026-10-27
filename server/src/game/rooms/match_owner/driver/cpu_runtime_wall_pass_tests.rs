use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{
        Intent, MemoryKey, MovementIntent, ObservedActor,
        policy::{ControlPlan, UtilityMemory},
        survival::SurvivalPlan,
    },
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, Equipment, PickupKind,
        Position, RulesConfig, SeatId, SpeedLevel, State, Terrain, World,
    },
    room::RoomSettings,
};
use tokio::time::Instant;

use super::tests::{publish_next_cpu_observation, running_driver_with_settings};

#[test]
fn real_driver_crosses_an_intact_block_and_preserves_collected_wall_pass() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 1, 180)
        .unwrap()
        .with_enabled_items(EnabledItems::WALL_PASS);
    let mut driver = running_driver_with_settings(23, settings);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(wall_pass_fixture(tick, settings));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    driver.cpu.as_mut().unwrap().cancel();
    let actor = cpu_actor(&basis);
    assert_eq!(actor.equipment, Some(Equipment::WallPass));
    let block = CellId::new(6, 7);
    assert!(basis.parts().arena.blocks.contains(&block));

    let key = MemoryKey {
        world: basis.parts().world,
        cpu_seat: SeatId::Two,
    };
    driver.cpu.as_mut().unwrap().inject_plan_result_for_test(
        basis,
        driver.runtime.lifecycle().round_attempt(),
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement: MovementIntent::Direction(Direction::West),
                    manual: None,
                },
                hold_ticks: 12,
                next_memory: UtilityMemory::new(key),
            },
            lease: None,
            execution: None,
        },
    );
    driver.poll_cpu_result().unwrap();

    let mut crossed = false;
    for _ in 0..14 {
        driver.tick(Instant::now()).unwrap();
        let Some(observation) = driver.last.cpu_observation.as_ref() else {
            continue;
        };
        let actor = cpu_actor(observation);
        assert_eq!(actor.equipment, Some(Equipment::WallPass));
        crossed |= actor.position.cell() == Some(block);
    }
    assert!(
        crossed,
        "ordinary Driver movement must enter the intact block"
    );
    let actual = &driver.runtime.sim().unwrap().state.actors()[SeatId::Two.index()];
    assert_eq!(actual.position().cell(), Some(block));
    assert_eq!(actual.inventory().equipment(), Some(Equipment::WallPass));

    let notice = driver.runtime.current_notice().unwrap();
    let frame = notice.frame(SeatId::One).expect("human public snapshot");
    assert!(!frame.contains("softWall"));
    assert!(!frame.contains("wallGrace"));
    let value: serde_json::Value = serde_json::from_str(&frame).unwrap();
    let cpu = value["state"]["actors"]
        .as_array()
        .unwrap()
        .iter()
        .find(|actor| actor["seat"] == "two")
        .unwrap();
    assert_eq!(cpu["inventory"]["equipment"], "wallPass");
    assert!(
        value["state"]["arena"]["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cell| cell == &serde_json::json!({"column": 6, "row": 7}))
    );
}

fn cpu_actor(
    observation: &grenadiers_core::cpu::Observation,
) -> &grenadiers_core::cpu::VisibleActor {
    match &observation.parts().actors[SeatId::Two.index()] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => panic!("CPU observes itself"),
    }
}

fn wall_pass_fixture(tick: grenadiers_core::game::Tick, settings: RoomSettings) -> State {
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
    let cpu_cell = CellId::new(7, 7);
    let arena = Arena::new(
        terrain,
        BTreeSet::from([CellId::new(6, 7)]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let actors = SeatId::DUEL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(if seat == SeatId::Two {
            cpu_cell
        } else {
            CellId::new(1, 1)
        }),
        heading: Direction::West,
        speed: SpeedLevel::Four,
    });
    State::new_at_tick(
        RulesConfig::expanded_v5(settings),
        World::with_pickups(
            arena,
            BTreeMap::new(),
            BTreeMap::new(),
            BTreeMap::from([(cpu_cell, PickupKind::WallPass)]),
        )
        .unwrap(),
        actors,
        tick,
    )
    .unwrap()
}
