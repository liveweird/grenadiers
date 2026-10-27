use std::{collections::BTreeMap, collections::BTreeSet, sync::Arc};

use grenadiers_core::{
    cpu::{BombKnowledge, ManualIntent, MovementIntent, Observation, ObservedActor},
    game::{
        ActorInit, Arena, BlastKind, CellId, Direction, EnabledItems, EntityId, Equipment,
        InputAction, PickupKind, Position, RulesConfig, SeatId, SeatInput, SpeedLevel, State,
        Terrain, Tick, World,
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
fn queued_driver_punches_the_same_bomb_id_and_survives_both_manual_outcomes() {
    for profile in [CpuProfile::CURRENT, CpuProfile::CURRENT_EASY] {
        run_punch_case(profile, false);
        run_punch_case(profile, true);
    }
}

fn run_punch_case(profile: CpuProfile, reject_manual: bool) {
    let (mut driver, planner, basis) = prepared_driver(profile);
    let bomb = sole_bomb(&basis);
    let source = bomb.cell;
    let bomb_id = bomb.id;
    let detonates_at = match bomb.trigger {
        grenadiers_core::cpu::ObservedBombTrigger::Timed {
            remaining_ticks,
            paused: false,
        } => Tick(basis.parts().tick.0 + u64::from(remaining_ticks)),
        _ => panic!("fixture bomb must be an unpaused timed bomb"),
    };
    assert_eq!(source, CellId::new(8, 7));
    assert_eq!(bomb.blast, BlastKind::Standard);
    assert!(basis.parts().pickups.is_empty());
    assert!(basis.parts().flames.is_empty());
    assert!(basis.parts().oil.is_empty());
    let cpu = visible_actor(&basis, SeatId::Two);
    let rival = visible_actor(&basis, SeatId::One);
    assert_eq!(cpu.position, Position::center(CellId::new(7, 7)));
    assert_eq!(cpu.heading, Direction::East);
    assert_eq!(cpu.equipment, Some(Equipment::Punch));
    assert!(rival.alive);
    assert_eq!(rival.equipment, None);

    let report = planner.pump_for_test(1);
    assert_eq!(report.dequeued, 1);
    assert_eq!(report.delivered, 1);
    assert_eq!(report.planned_intents.len(), 1);
    assert_eq!(
        report.planned_intents[0].movement,
        MovementIntent::Stop,
        "{report:?}"
    );
    assert_eq!(
        report.planned_intents[0].manual,
        Some(ManualIntent::Equipment)
    );

    let admission = Tick(basis.parts().tick.0 + 6);
    settle_to(&mut driver, admission);
    if reject_manual {
        let epoch = driver.runtime.lifecycle().round_attempt();
        for _ in 0..7 {
            driver
                .runtime
                .action(SeatId::Two, epoch, MatchAction::Movement(None))
                .unwrap();
        }
    }
    let manual_before = driver.runtime.next_cpu_manual_action();
    driver.poll_cpu_result().unwrap();
    let manual_after = driver.runtime.next_cpu_manual_action();
    assert_eq!(driver.runtime.lifecycle().tick(), admission);
    assert!(driver.cpu_stationary);
    assert!(
        !driver.cpu.as_ref().unwrap().is_escaping(),
        "accepted Equipment must not enter the legacy Bomb escape executor"
    );
    assert!(driver.cpu.as_ref().unwrap().has_committed_route());
    if reject_manual {
        assert_eq!(manual_after, manual_before);
    } else {
        assert_eq!(manual_after, manual_before.and_then(|id| id.checked_add(1)));
    }

    let expected_cell = if reject_manual {
        source
    } else {
        CellId::new(11, 7)
    };
    let mut confirmed = false;
    let mut saw_flame = false;
    let mut survived_clear = false;
    let mut last_alive = true;
    let mut last_observation_tick = basis.parts().tick;
    let clear_by = Tick(detonates_at.0 + 33);
    while driver.runtime.lifecycle().tick() <= clear_by {
        driver.tick(Instant::now()).unwrap();
        let Some(observation) = driver.last.cpu_observation.as_ref() else {
            continue;
        };
        if !confirmed && observation.parts().tick >= Tick(basis.parts().tick.0 + 7) {
            let current = sole_bomb(observation);
            assert_eq!(
                current.id, bomb_id,
                "Punch must preserve physical bomb identity"
            );
            assert_eq!(current.cell, expected_cell);
            confirmed = true;
        }
        let actor = visible_actor(observation, SeatId::Two);
        last_alive = actor.alive;
        last_observation_tick = observation.parts().tick;
        assert_eq!(actor.equipment, Some(Equipment::Punch));
        if !observation.parts().flames.is_empty() {
            saw_flame = true;
            assert!(actor.alive, "committed common route must survive its bomb");
        } else if saw_flame {
            assert!(
                matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
            );
            survived_clear = actor.alive;
        }
    }
    assert!(confirmed);
    assert!(saw_flame);
    assert!(
        survived_clear,
        "CPU must survive full fuse and flame expiry: reject={reject_manual} saw={saw_flame} lastTick={last_observation_tick:?} alive={last_alive} phase={:?}",
        driver.runtime.lifecycle().phase()
    );
    assert_eq!(driver.runtime.next_cpu_manual_action(), manual_after);
}

fn prepared_driver(profile: CpuProfile) -> (Driver, CpuPlanner, Observation) {
    let enabled = EnabledItems::from_bits(
        EnabledItems::PUNCH.bits() | EnabledItems::KICK.bits() | EnabledItems::TELEPORTER.bits(),
    )
    .unwrap();
    let settings = RoomSettings::new(SpeedLevel::Four, 64, 1, 180)
        .unwrap()
        .with_enabled_items(enabled);
    let (mut driver, planner) = running_driver(settings, profile);
    let tick = driver.runtime.lifecycle().tick();
    driver
        .runtime
        .replace_state_for_test(punch_fixture(tick, settings));
    let initial = driver.runtime.current_notice().unwrap();
    driver.consider_cpu_observation(&initial);
    let basis = publish_next_cpu_observation(&mut driver);
    (driver, planner, basis)
}

fn settle_to(driver: &mut Driver, tick: Tick) {
    while driver.runtime.lifecycle().tick() < tick {
        if let Some(notice) = driver.runtime.settle_boundary([None; 2]).unwrap() {
            driver.publish(notice);
        }
    }
}

fn sole_bomb(observation: &Observation) -> &grenadiers_core::cpu::ObservedBomb {
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        panic!("fixture bomb knowledge must be visible")
    };
    assert_eq!(bombs.len(), 1);
    &bombs[0]
}

fn visible_actor(observation: &Observation, seat: SeatId) -> &grenadiers_core::cpu::VisibleActor {
    match &observation.parts().actors[seat.index()] {
        ObservedActor::Visible(actor) => actor,
        ObservedActor::Hidden { .. } => panic!("fixture actors must be visible"),
    }
}

fn running_driver(settings: RoomSettings, profile: CpuProfile) -> (Driver, CpuPlanner) {
    let (_, shutdown) = watch::channel(false);
    let mut room = identity(RoomProfile::ExpandedV5);
    room.mode = RoomMode::Solo(profile);
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
    driver.runtime.cpu_ready().unwrap();
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

fn punch_fixture(tick: Tick, settings: RoomSettings) -> State {
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
        BTreeMap::from([
            (CellId::new(13, 11), PickupKind::Punch),
            (CellId::new(6, 7), PickupKind::Teleporter),
        ]),
    )
    .unwrap();
    let mut state = State::new_at_tick(
        RulesConfig::expanded_v5(settings),
        world,
        [
            ActorInit {
                id: EntityId(1),
                seat: SeatId::One,
                position: Position::center(CellId::new(8, 7)),
                heading: Direction::West,
                speed: SpeedLevel::Four,
            },
            ActorInit {
                id: EntityId(2),
                seat: SeatId::Two,
                position: Position::center(CellId::new(13, 11)),
                heading: Direction::East,
                speed: SpeedLevel::Four,
            },
        ],
        Tick(tick.0 - 28),
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
            action: InputAction::Press(Direction::West),
        }])
        .unwrap();
    for _ in 0..18 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[
            SeatInput {
                seat: SeatId::One,
                action: InputAction::Release(Direction::West),
            },
            SeatInput {
                seat: SeatId::Two,
                action: InputAction::Press(Direction::East),
            },
        ])
        .unwrap();
    for _ in 0..6 {
        state.step(&[]).unwrap();
    }
    state
        .step(&[SeatInput {
            seat: SeatId::Two,
            action: InputAction::Release(Direction::East),
        }])
        .unwrap();
    assert_eq!(state.tick(), tick);
    assert_eq!(
        state.actors()[SeatId::One.index()].position(),
        Position::center(CellId::new(13, 11))
    );
    assert_eq!(
        state.actors()[SeatId::Two.index()].position(),
        Position::center(cpu_cell)
    );
    assert_eq!(
        state.actors()[SeatId::Two.index()].inventory().equipment(),
        Some(Equipment::Punch)
    );
    assert_eq!(state.world().bombs().len(), 1);
    assert_eq!(state.world().bombs().values().next().unwrap().range(), 2);
    assert_eq!(
        state.world().bombs().values().next().unwrap().cell(),
        CellId::new(8, 7)
    );
    assert!(state.world().exposed_pickups().is_empty());
    assert!(state.world().hazards().is_empty());
    state
}
