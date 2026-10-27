use std::{collections::BTreeMap, collections::BTreeSet, sync::Arc};

use grenadiers_core::{
    cpu::{BombKnowledge, ObservedActor},
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, Equipment, InputAction,
        PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State, Terrain, Tick,
        World,
    },
    room::{CpuProfile, MatchPhase, RoomMode, RoomSettings},
};
use tokio::{sync::watch, time::Instant};

use super::super::Driver;
use super::tests::publish_next_cpu_observation;
use crate::{
    game::rooms::{
        cpu::CpuPlanner,
        match_owner::{PreparedMatchOwner, test_support::identity},
        match_runtime::MatchAction,
        profile::RoomProfile,
    },
    round_seed::RoundSeedSource,
};

#[test]
fn queued_driver_uses_a_collected_bracelet_to_shove_a_rival_into_fire() {
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 1, 60)
        .unwrap()
        .with_enabled_items(EnabledItems::BRACELET);
    let (mut driver, planner, basis) = prepared_tactic_driver(settings, false);
    let (mut cooling_driver, cooling_planner, cooling_basis) =
        prepared_tactic_driver(settings, true);
    let cpu = visible_actor(&basis, SeatId::Two);
    let rival = visible_actor(&basis, SeatId::One);
    assert_eq!(cpu.position, Position::center(CellId::new(7, 7)));
    assert_eq!(cpu.equipment, Some(Equipment::Bracelet));
    assert_eq!(rival.position, Position::center(CellId::new(8, 7)));
    assert!(rival.alive);
    assert!(matches!(&basis.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(basis.parts().pickups.is_empty());
    assert!(
        basis
            .parts()
            .flames
            .iter()
            .any(|flame| flame.cell == CellId::new(9, 7))
    );
    assert_eq!(
        cooling_basis, basis,
        "private Bracelet cooldown must not alter the public CPU observation"
    );

    assert_bracelet_plan(&planner);
    assert_bracelet_plan(&cooling_planner);
    driver.poll_cpu_result().unwrap();
    cooling_driver.poll_cpu_result().unwrap();
    let admission_tick = driver.runtime.lifecycle().tick();
    for candidate in [&driver, &cooling_driver] {
        assert_eq!(
            candidate.cpu_stop_at,
            Some(Tick(admission_tick.0 + 12)),
            "Bracelet tactic must use one ordinary 12-tick movement hold"
        );
        assert!(!candidate.cpu_stationary);
    }

    for _ in 0..12 {
        driver.tick(Instant::now()).unwrap();
        cooling_driver.tick(Instant::now()).unwrap();
        let state = &driver.runtime.sim().unwrap().state;
        if !state.actors()[SeatId::One.index()].alive() {
            break;
        }
    }

    let state = &driver.runtime.sim().unwrap().state;
    assert!(
        !state.actors()[SeatId::One.index()].alive(),
        "the shove must carry the rival into the existing fire"
    );
    assert!(state.actors()[SeatId::Two.index()].alive());
    assert!(
        state.world().bombs().is_empty(),
        "the tactic must not plant"
    );
    let cooling = &cooling_driver.runtime.sim().unwrap().state;
    assert!(
        cooling.actors()[SeatId::One.index()].alive(),
        "private cooldown may turn the same public shove tactic into movement without a shove"
    );
    assert!(cooling.actors()[SeatId::Two.index()].alive());
    assert_ne!(
        cooling.actors()[SeatId::Two.index()].position().cell(),
        Some(CellId::new(9, 7)),
        "the same admitted path must remain survivable when the shove is unavailable"
    );
    assert!(cooling.world().bombs().is_empty());
}

fn prepared_tactic_driver(
    settings: RoomSettings,
    cooling_down: bool,
) -> (Driver, CpuPlanner, grenadiers_core::cpu::Observation) {
    let (mut driver, planner) = running_driver_with_queued_planner(settings);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(bracelet_fire_fixture(tick, settings, cooling_down));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    (driver, planner, basis)
}

fn assert_bracelet_plan(planner: &CpuPlanner) {
    let report = planner.pump_for_test(1);
    assert_eq!(report.dequeued, 1);
    assert_eq!(report.delivered, 1);
    assert_eq!(report.planned_intents.len(), 1);
    assert_eq!(report.planned_intents[0].manual, None);
    assert_eq!(
        report.planned_intents[0].movement,
        grenadiers_core::cpu::MovementIntent::Direction(Direction::East)
    );
}

fn visible_actor(
    observation: &grenadiers_core::cpu::Observation,
    seat: SeatId,
) -> &grenadiers_core::cpu::VisibleActor {
    match &observation.parts().actors[seat.index()] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => panic!("fixture actor must be visible"),
    }
}

fn running_driver_with_queued_planner(settings: RoomSettings) -> (Driver, CpuPlanner) {
    let (_, shutdown) = watch::channel(false);
    let mut room = identity(RoomProfile::ExpandedV5);
    room.mode = RoomMode::Solo(CpuProfile::CURRENT);
    room.private_cpu_seed = Some([17; 32]);
    let planner = CpuPlanner::without_workers();
    let (_, owner) = PreparedMatchOwner::new_with_planner(
        room,
        Instant::now(),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(23);
            Ok(())
        }),
        Arc::new(|bytes: &mut [u8]| {
            bytes.fill(31);
            Ok(())
        }),
        planner.clone(),
        shutdown,
    )
    .unwrap();
    let mut driver = Driver::new(
        owner.runtime,
        owner.initial,
        owner.notices,
        Instant::now(),
        owner.random_fill,
        owner.cpu,
    );
    let creator = driver.runtime.prepare_solo_creator(SeatId::One).unwrap();
    driver.runtime.commit_admission(creator);
    driver
        .runtime
        .action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            MatchAction::UpdateSettings {
                settings_revision: 1,
                settings,
            },
        )
        .unwrap();
    driver.runtime.cpu_ready(SeatId::Two).unwrap();
    driver
        .runtime
        .action(
            SeatId::One,
            driver.runtime.lifecycle().round_attempt(),
            MatchAction::ReadyChecked {
                ready: true,
                settings_revision: 2,
            },
        )
        .unwrap();
    while driver.runtime.lifecycle().phase() != MatchPhase::Running {
        driver.runtime.settle_boundary([None; 2]).unwrap();
    }
    (driver, planner)
}

fn bracelet_fire_fixture(tick: Tick, settings: RoomSettings, cooling_down: bool) -> State {
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
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(cpu_cell, PickupKind::Bracelet)]),
    )
    .unwrap();
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(settings),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position {
                    x: Position::center(CellId::new(11, 7)).x - i32::from(cooling_down),
                    ..Position::center(CellId::new(11, 7))
                },
                heading: Direction::West,
                speed: SpeedLevel::Four,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(cpu_cell),
                heading: Direction::East,
                speed: SpeedLevel::Four,
            },
        ],
        Tick(tick.0 - 61),
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
                action: InputAction::Press(Direction::West),
            },
        ])
        .unwrap();
    if cooling_down {
        for _ in 0..38 {
            state.step(&[]).unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            }])
            .unwrap();
        state
            .step(&[
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Release(Direction::East),
                },
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Press(Direction::West),
                },
            ])
            .unwrap();
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Release(Direction::West),
            }])
            .unwrap();
        for _ in 0..6 {
            state.step(&[]).unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::West),
            }])
            .unwrap();
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::West),
            }])
            .unwrap();
        for _ in 0..2 {
            state.step(&[]).unwrap();
        }
        state
            .step(&[
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Release(Direction::West),
                },
                SeatInput {
                    seat: SeatId::Two,
                    action: InputAction::Press(Direction::East),
                },
            ])
            .unwrap();
        for _ in 0..2 {
            state.step(&[]).unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::Two,
                action: InputAction::Release(Direction::East),
            }])
            .unwrap();
        for _ in 0..5 {
            state.step(&[]).unwrap();
        }
    } else {
        for _ in 0..35 {
            state.step(&[]).unwrap();
        }
        state
            .step(&[SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::West),
            }])
            .unwrap();
        for _ in 0..24 {
            state.step(&[]).unwrap();
        }
    }
    assert_eq!(state.tick(), tick);
    assert_eq!(
        state.actors()[SeatId::One.index()].position(),
        Position::center(CellId::new(8, 7))
    );
    assert_eq!(
        state.actors()[SeatId::Two.index()].inventory().equipment(),
        Some(Equipment::Bracelet)
    );
    assert!(state.world().bombs().is_empty());
    assert!(state.world().exposed_pickups().is_empty());
    assert!(
        state
            .world()
            .hazards()
            .get(&CellId::new(9, 7))
            .is_some_and(|expiry| *expiry > tick)
    );
    state
}
