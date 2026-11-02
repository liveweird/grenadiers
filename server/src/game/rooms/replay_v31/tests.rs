use grenadiers_core::{
    game::{
        BombTrigger, CellId, Direction, EnabledItems, Equipment, PunchActivationPolicy, SeatId,
        StormCloudConfig,
    },
    room::{
        BoardSize, CpuDifficulty, LairRoomConfiguration, RespawnLairTuning, RoomConfiguration,
        RoomSettings, SeatControl, SpikePitTuning, SpikeRoomConfiguration,
    },
};

use super::*;
use crate::round_seed::RoundSeedSource;

fn configuration(difficulty: Option<CpuDifficulty>) -> SpikeRoomConfiguration {
    let controllers = if difficulty.is_some() {
        vec![SeatControl::Human, SeatControl::Cpu]
    } else {
        vec![SeatControl::Human, SeatControl::Human]
    };
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(&controllers, difficulty, BoardSize::FifteenByThirteen).unwrap(),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT.as_legacy(),
        ),
        SpikePitTuning::DEFAULT,
    )
}

fn recorder(difficulty: Option<CpuDifficulty>) -> RecordingRosterRuntime {
    recorder_with_items(difficulty, EnabledItems::ALL)
}

fn recorder_with_items(
    difficulty: Option<CpuDifficulty>,
    enabled_items: EnabledItems,
) -> RecordingRosterRuntime {
    let cpus = difficulty
        .map(|difficulty| vec![(SeatId::Two, difficulty, [31; 32])])
        .unwrap_or_default();
    RecordingRosterRuntime::new(
        configuration(difficulty),
        RoomSettings::DEFAULT.with_enabled_items(enabled_items),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(31);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

fn move_cells(recording: &mut RecordingRosterRuntime, direction: Direction, cells: usize) {
    recording
        .human_action(SeatId::One, 1, ReplayAction::Movement(Some(direction)))
        .unwrap();
    for update in 0..cells * 20 {
        if update > 0 && update.is_multiple_of(10) {
            recording
                .human_action(SeatId::One, 1, ReplayAction::Movement(Some(direction)))
                .unwrap();
        }
        recording.settle([None; 4]).unwrap();
    }
    recording
        .human_action(SeatId::One, 1, ReplayAction::Movement(None))
        .unwrap();
    recording.settle([None; 4]).unwrap();
}

fn plant(recording: &mut RecordingRosterRuntime, action_id: u32) {
    recording
        .human_action(SeatId::One, 1, ReplayAction::Bomb(action_id))
        .unwrap();
    recording.settle([None; 4]).unwrap();
}

fn wait_clear(recording: &mut RecordingRosterRuntime) {
    for _ in 0..240 {
        let inspection = recording.runtime().replay_inspection();
        let world = inspection.simulation.unwrap().state;
        if world.world().bombs().is_empty() && world.world().hazards().is_empty() {
            return;
        }
        recording.settle([None; 4]).unwrap();
    }
    panic!("recorded hazards failed to clear");
}

fn start(recording: &mut RecordingRosterRuntime) {
    let humans: Vec<_> = recording
        .runtime()
        .identity()
        .configuration
        .human_seats()
        .collect();
    for seat in humans {
        recording.admit(seat, false).unwrap();
        recording
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay31_binds_contact_v2_and_exact_current_policy_pairs() {
    for (difficulty, policy) in [
        (None, 60),
        (Some(CpuDifficulty::Easy), 60),
        (Some(CpuDifficulty::Standard), 60),
        (Some(CpuDifficulty::Hard), 61),
    ] {
        let mut recording = recorder(difficulty);
        start(&mut recording);
        let expected = recording.runtime().replay_inspection();
        let (_, journal) = recording.into_parts();
        assert_eq!(journal.header.format_version, 31);
        assert_eq!(journal.header.gameplay_version, 20);
        assert_eq!(journal.header.rules_version, 10);
        assert_eq!(journal.header.mob_navigation_version, 3);
        assert_eq!(journal.header.punch_activation_version, 2);
        assert_eq!(journal.header.cpu_policy_version, policy);
        assert_eq!(expected.punch_activation, PunchActivationPolicy::ContactV2);
        verify_exact(&journal, &expected).unwrap();
    }
}

#[test]
fn replay31_rejects_manual_activation_and_archived_or_mixed_policy_pairs() {
    let (_, journal) = recorder(Some(CpuDifficulty::Hard)).into_parts();
    let mut manual = journal.clone();
    manual.header.punch_activation_version = 1;
    assert_eq!(replay(&manual), Err(ReplayError::InvalidHeader));

    for wrong in [56, 58, 59, 60] {
        let mut bad = journal.clone();
        bad.header.cpu_policy_version = wrong;
        bad.body.header.cpu_policy_version = wrong;
        bad.header.cpus[0].policy_version = wrong;
        bad.body.header.cpus[0].policy_version = wrong;
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }

    let (_, standard) = recorder(Some(CpuDifficulty::Standard)).into_parts();
    let mut hard = standard.clone();
    hard.header.cpu_policy_version = 61;
    hard.body.header.cpu_policy_version = 61;
    hard.header.cpus[0].policy_version = 61;
    hard.body.header.cpus[0].policy_version = 61;
    assert_eq!(replay(&hard), Err(ReplayError::InvalidHeader));
}

#[test]
fn replay31_rejects_human_equipment_actions_even_with_valid_counters() {
    use replay_v8::record::{ActionSource, Operation};

    let mut recording = recorder(None);
    start(&mut recording);
    let (_, journal) = recording.into_parts();
    assert!(replay(&journal).is_ok());
    for id in [1, grenadiers_core::protocol::MAX_COUNTER] {
        let mut forged = journal.clone();
        let record = forged
            .body
            .records
            .iter_mut()
            .find(|record| {
                matches!(
                    record.operation,
                    Operation::Action {
                        source: ActionSource::Human,
                        ..
                    }
                )
            })
            .unwrap();
        let Operation::Action { action, .. } = &mut record.operation else {
            unreachable!()
        };
        *action = ReplayAction::Equipment(id);
        assert_eq!(replay(&forged), Err(ReplayError::InvalidHeader));
    }
}

#[test]
fn replay31_reconstructs_an_actual_contact_punch_without_an_equipment_action() {
    let mut recording = recorder_with_items(None, EnabledItems::PUNCH);
    start(&mut recording);

    move_cells(&mut recording, Direction::East, 1);
    plant(&mut recording, 1);
    move_cells(&mut recording, Direction::West, 1);
    move_cells(&mut recording, Direction::South, 1);
    wait_clear(&mut recording);
    move_cells(&mut recording, Direction::North, 1);
    move_cells(&mut recording, Direction::East, 2);
    assert_eq!(
        recording
            .runtime()
            .replay_inspection()
            .simulation
            .unwrap()
            .state
            .actors()[0]
            .inventory()
            .equipment(),
        Some(Equipment::Punch)
    );

    plant(&mut recording, 2);
    move_cells(&mut recording, Direction::West, 2);
    move_cells(&mut recording, Direction::South, 1);
    wait_clear(&mut recording);
    move_cells(&mut recording, Direction::North, 1);
    move_cells(&mut recording, Direction::East, 3);
    plant(&mut recording, 3);
    move_cells(&mut recording, Direction::West, 1);
    move_cells(&mut recording, Direction::South, 1);
    wait_clear(&mut recording);
    move_cells(&mut recording, Direction::North, 1);
    move_cells(&mut recording, Direction::East, 1);
    plant(&mut recording, 4);
    let planted = {
        let state = &recording
            .runtime()
            .replay_inspection()
            .simulation
            .unwrap()
            .state;
        let owner = state.actors()[0].id();
        *state
            .world()
            .bombs()
            .values()
            .find(|bomb| bomb.owner() == owner && bomb.cell() == CellId::new(4, 1))
            .expect("the named punch witness bomb must be planted at its origin")
    };
    assert!(!planted.has_been_punched());
    assert!(matches!(planted.trigger(), BombTrigger::Timed { .. }));
    move_cells(&mut recording, Direction::East, 1);

    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    let mut witnessed_punch = false;
    for _ in 0..20 {
        recording.settle([None; 4]).unwrap();
        let inspection = recording.runtime().replay_inspection();
        if inspection
            .simulation
            .unwrap()
            .state
            .world()
            .bombs()
            .get(&planted.id())
            .is_some_and(|bomb| bomb.has_been_punched() && bomb.cell() == CellId::new(1, 1))
        {
            witnessed_punch = true;
            break;
        }
    }
    assert!(
        witnessed_punch,
        "the named bomb must land at the fixed cell; observed {:?}",
        recording
            .runtime()
            .replay_inspection()
            .simulation
            .unwrap()
            .state
            .world()
            .bombs()
            .get(&planted.id())
    );
    recording
        .human_action(SeatId::One, 1, ReplayAction::Movement(None))
        .unwrap();
    recording.settle([None; 4]).unwrap();

    let expected = recording.runtime().replay_inspection();
    let state = &expected.simulation.as_ref().unwrap().state;
    let punched = state
        .world()
        .bombs()
        .get(&planted.id())
        .expect("the named punched bomb must remain live for replay inspection");
    assert_eq!(punched.cell(), CellId::new(1, 1));
    assert_eq!(punched.owner(), planted.owner());
    assert_eq!(punched.placed_at(), planted.placed_at());
    assert_eq!(punched.trigger(), planted.trigger());
    assert!(punched.has_been_punched());
    let (_, journal) = recording.into_parts();
    assert!(journal.body.records.iter().any(|record| matches!(
        record.operation,
        replay_v8::record::Operation::Action {
            action: ReplayAction::Movement(Some(Direction::West)),
            ..
        }
    )));
    assert!(!journal.body.records.iter().any(|record| matches!(
        record.operation,
        replay_v8::record::Operation::Action {
            action: ReplayAction::Equipment(_),
            ..
        }
    )));
    verify_exact(&journal, &expected).unwrap();
}
