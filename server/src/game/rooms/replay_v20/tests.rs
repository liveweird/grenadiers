use grenadiers_core::{
    game::{EnabledItems, SeatId, StormCloudConfig},
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
    let cpus = difficulty
        .map(|difficulty| vec![(SeatId::Two, difficulty, [20; 32])])
        .unwrap_or_default();
    RecordingRosterRuntime::new(
        configuration(difficulty),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(20);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

#[test]
fn replay20_binds_exact_current_policy_pairs_and_reconstructs() {
    for (difficulty, policy) in [
        (None, 40),
        (Some(CpuDifficulty::Easy), 40),
        (Some(CpuDifficulty::Standard), 40),
        (Some(CpuDifficulty::Hard), 41),
    ] {
        let mut recording = recorder(difficulty);
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
        let expected = recording.runtime().replay_inspection();
        let (_, journal) = recording.into_parts();
        assert_eq!(journal.header.format_version, 20);
        assert_eq!(journal.header.gameplay_version, 19);
        assert_eq!(journal.header.rules_version, 10);
        assert_eq!(journal.header.mob_navigation_version, 3);
        assert_eq!(journal.header.cpu_policy_version, policy);
        verify_exact(&journal, &expected).unwrap();
    }
}

#[test]
fn replay20_rejects_archived_policy_pairs() {
    for (difficulty, wrong) in [
        (CpuDifficulty::Standard, 41),
        (CpuDifficulty::Hard, 37),
        (CpuDifficulty::Hard, 39),
        (CpuDifficulty::Hard, 40),
    ] {
        let (_, journal) = recorder(Some(difficulty)).into_parts();
        let mut bad = journal.clone();
        bad.header.cpu_policy_version = wrong;
        bad.body.header.cpu_policy_version = wrong;
        bad.header.cpus[0].policy_version = wrong;
        bad.body.header.cpus[0].policy_version = wrong;
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }
}

#[test]
fn replay20_rejects_future_blocked_birth_policy_pairs() {
    for (difficulty, future) in [
        (CpuDifficulty::Easy, 42),
        (CpuDifficulty::Standard, 42),
        (CpuDifficulty::Hard, 43),
    ] {
        let (_, journal) = recorder(Some(difficulty)).into_parts();
        let mut bad = journal.clone();
        bad.header.cpu_policy_version = future;
        bad.body.header.cpu_policy_version = future;
        bad.header.cpus[0].policy_version = future;
        bad.body.header.cpus[0].policy_version = future;
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }
}
