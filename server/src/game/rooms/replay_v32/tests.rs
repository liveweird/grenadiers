use grenadiers_core::{
    game::{ArenaConstructionProfile, Direction, PunchActivationPolicy, SeatId, StormCloudConfig},
    room::{
        BoardSize, CpuDifficulty, LairRoomConfiguration, RespawnLairTuning, RoomConfiguration,
        RoomSettings, SeatControl, SpikePitTuning, SpikeRoomConfiguration,
    },
};

use super::*;
use crate::round_seed::RoundSeedSource;

fn configuration(difficulty: Option<CpuDifficulty>, board: BoardSize) -> SpikeRoomConfiguration {
    let controllers = if difficulty.is_some() {
        vec![SeatControl::Human, SeatControl::Cpu]
    } else {
        vec![SeatControl::Human, SeatControl::Human]
    };
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(&controllers, difficulty, board).unwrap(),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT.as_legacy(),
        ),
        SpikePitTuning::DEFAULT,
    )
}

fn recorder(difficulty: Option<CpuDifficulty>, board: BoardSize) -> RecordingRosterRuntime {
    let cpus = difficulty
        .map(|difficulty| vec![(SeatId::Two, difficulty, [32; 32])])
        .unwrap_or_default();
    RecordingRosterRuntime::new(
        configuration(difficulty, board),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(32);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
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
fn replay32_binds_clearance_profile_and_current_policy_pairs_on_every_board() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        for (difficulty, policy) in [
            (None, 62),
            (Some(CpuDifficulty::Easy), 62),
            (Some(CpuDifficulty::Standard), 62),
            (Some(CpuDifficulty::Hard), 64),
        ] {
            let mut recording = recorder(difficulty, board);
            start(&mut recording);
            let expected = recording.runtime().replay_inspection();
            let (_, journal) = recording.into_parts();
            assert_eq!(journal.header.format_version, 32);
            assert_eq!(journal.header.generator_version, 5);
            assert_eq!(journal.header.spike_pit_layout_version, 2);
            assert_eq!(journal.header.cpu_policy_version, policy);
            assert_eq!(expected.construction_profile, CONSTRUCTION_PROFILE);
            assert_eq!(expected.punch_activation, PunchActivationPolicy::ContactV2);
            verify_exact(&journal, &expected).unwrap();
        }
    }
}

#[test]
fn replay32_rejects_archived_or_mixed_construction_identities() {
    let (_, journal) =
        recorder(Some(CpuDifficulty::Hard), BoardSize::FifteenByThirteen).into_parts();
    for (generator, pits) in [(3, 2), (5, 1), (4, 2)] {
        let mut bad = journal.clone();
        bad.header.generator_version = generator;
        bad.header.spike_pit_layout_version = pits;
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }
    let mut manual = journal.clone();
    manual.header.punch_activation_version = 1;
    assert_eq!(replay(&manual), Err(ReplayError::InvalidHeader));

    let mut old_policy = journal;
    old_policy.header.cpu_policy_version = 59;
    old_policy.body.header.cpu_policy_version = 59;
    old_policy.header.cpus[0].policy_version = 59;
    old_policy.body.header.cpus[0].policy_version = 59;
    assert_eq!(replay(&old_policy), Err(ReplayError::InvalidHeader));
}

#[test]
fn replay32_reconstructs_accepted_input_and_complete_profiled_state() {
    let mut recording = recorder(None, BoardSize::FifteenByThirteen);
    start(&mut recording);
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    recording.settle([None; 4]).unwrap();
    recording
        .human_action(SeatId::One, 1, ReplayAction::Movement(None))
        .unwrap();
    recording
        .human_action(SeatId::One, 1, ReplayAction::Bomb(1))
        .unwrap();
    recording.settle([None; 4]).unwrap();

    let expected = recording.runtime().replay_inspection();
    assert_eq!(
        expected.construction_profile,
        ArenaConstructionProfile::LairClearanceV5
    );
    let (_, journal) = recording.into_parts();
    verify_exact(&journal, &expected).unwrap();
}
