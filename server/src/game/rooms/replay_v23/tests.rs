use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId, StormCloudConfig},
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
        .map(|difficulty| vec![(SeatId::Two, difficulty, [22; 32])])
        .unwrap_or_default();
    RecordingRosterRuntime::new(
        configuration(difficulty),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(22);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

#[test]
fn replay23_binds_exact_current_policy_pairs_and_reconstructs() {
    for (difficulty, policy) in [
        (
            None,
            grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                .policy
                .version(),
        ),
        (
            Some(CpuDifficulty::Easy),
            grenadiers_core::room::CpuProfile::TELEPORTER_V46_EASY
                .policy
                .version(),
        ),
        (
            Some(CpuDifficulty::Standard),
            grenadiers_core::room::CpuProfile::TELEPORTER_V46_STANDARD
                .policy
                .version(),
        ),
        (
            Some(CpuDifficulty::Hard),
            grenadiers_core::room::CpuProfile::HARD_V47.policy.version(),
        ),
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
        assert_eq!(journal.header.format_version, 23);
        assert_eq!(journal.header.gameplay_version, 19);
        assert_eq!(journal.header.rules_version, 10);
        assert_eq!(journal.header.mob_navigation_version, 3);
        assert_eq!(journal.header.cpu_policy_version, policy);
        verify_exact(&journal, &expected).unwrap();
    }
}

#[test]
fn replay23_rejects_archived_policy_pairs() {
    for (difficulty, wrong) in [
        (CpuDifficulty::Standard, 44),
        (CpuDifficulty::Hard, 45),
        (CpuDifficulty::Hard, 44),
        (CpuDifficulty::Hard, 46),
        (CpuDifficulty::Standard, 48),
        (CpuDifficulty::Hard, 49),
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
fn replay23_preserves_cpu_input_fence_action_order() {
    use replay_v8::record::{ActionSource, Operation};

    let mut recording = recorder(Some(CpuDifficulty::Standard));
    recording.admit(SeatId::One, false).unwrap();
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }
    recording
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(Direction::East)),
        )
        .unwrap();
    recording.settle([None; 4]).unwrap();
    recording.input_fence(SeatId::Two);
    recording
        .cpu_action(
            SeatId::Two,
            1,
            ReplayAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    recording.settle([None; 4]).unwrap();

    let expected = recording.runtime().replay_inspection();
    let (_, journal) = recording.into_parts();
    let ordered = journal
        .body
        .records
        .iter()
        .filter_map(|record| match record.operation {
            Operation::Action {
                seat: SeatId::Two,
                source: ActionSource::Cpu,
                action: ReplayAction::Movement(direction),
                ..
            } => Some(Some(direction)),
            Operation::InputFence(SeatId::Two) => Some(None),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        ordered,
        vec![
            Some(Some(Direction::East)),
            None,
            Some(Some(Direction::West))
        ]
    );
    verify_exact(&journal, &expected).unwrap();
}
