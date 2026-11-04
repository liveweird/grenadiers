use grenadiers_core::{
    game::{
        ArenaConstructionProfile, EnabledItems, PunchActivationPolicy, SeatId, StormCloudConfig,
    },
    room::{
        BombCapacityDropMultiplier, CpuDifficulty, LairRoomConfiguration, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
        SpikeRoomConfiguration,
    },
};

use super::*;
use crate::round_seed::RoundSeedSource;

fn weighted(value: u64) -> RoomSettings {
    RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::new(value).unwrap())
}

fn configuration(difficulty: Option<CpuDifficulty>) -> SpikeRoomConfiguration {
    let controllers = if difficulty.is_some() {
        vec![SeatControl::Human, SeatControl::Cpu]
    } else {
        vec![SeatControl::Human, SeatControl::Human]
    };
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(
                &controllers,
                difficulty,
                grenadiers_core::room::BoardSize::FifteenByThirteen,
            )
            .unwrap(),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT.as_legacy(),
        ),
        SpikePitTuning::new(true, SpikePitSettings::new(36, 180).unwrap()),
    )
}

fn recorder(difficulty: Option<CpuDifficulty>, multiplier: u64) -> RecordingRosterRuntime {
    let cpus = difficulty
        .map(|difficulty| vec![(SeatId::Two, difficulty, [33; 32])])
        .unwrap_or_default();
    RecordingRosterRuntime::new(
        configuration(difficulty),
        weighted(multiplier),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(33);
            Ok(())
        }),
        cpus,
    )
    .unwrap()
}

fn ready_humans(recording: &mut RecordingRosterRuntime, epoch: u64, revision: u32) {
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
                epoch,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: revision,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay33_reconstructs_deterministic_settings_actions_and_rounds() {
    let mut recording = recorder(None, 5);
    recording.admit(SeatId::One, false).unwrap();
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSpikeSettings {
                settings_revision: 1,
                settings: weighted(10),
                cloud: StormCloudConfig::Single,
                tuning: RespawnLairTuning::DEFAULT,
                spike_pits: SpikePitTuning::new(true, SpikePitSettings::new(36, 180).unwrap()),
            },
        )
        .unwrap();
    recording.admit(SeatId::Two, false).unwrap();
    for seat in SeatId::DUEL {
        recording
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }
    recording
        .interrupt(grenadiers_core::room::InterruptReason::ServerStopped)
        .unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        recording.settle([None; 4]).unwrap();
    }
    for seat in SeatId::DUEL {
        recording
            .human_action(seat, 1, ReplayAction::Rematch(true))
            .unwrap();
    }
    for seat in SeatId::DUEL {
        recording
            .human_action(
                seat,
                2,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }

    let expected = recording.runtime().replay_inspection();
    assert_eq!(expected.simulation.as_ref().unwrap().epoch, 2);
    assert_eq!(
        expected.simulation.as_ref().unwrap().generation.version(),
        7
    );
    assert_eq!(
        expected.construction_profile,
        ArenaConstructionProfile::WeightedLairClearanceV7
    );
    assert_eq!(expected.punch_activation, PunchActivationPolicy::ContactV2);
    assert_eq!(
        expected
            .lifecycle
            .settings()
            .bomb_capacity_drop_multiplier(),
        BombCapacityDropMultiplier::new(10).unwrap()
    );
    let (_, journal) = recording.into_parts();
    assert_eq!(journal.header.format_version, 33);
    verify_exact(&journal, &expected).unwrap();
}

#[test]
fn replay33_accepts_multiplier_extrema_and_rejects_header_body_disagreement() {
    for multiplier in [1, 10] {
        let mut recording = recorder(None, multiplier);
        ready_humans(&mut recording, 1, 1);
        let expected = recording.runtime().replay_inspection();
        let (_, journal) = recording.into_parts();
        verify_exact(&journal, &expected).unwrap();

        let mut bad = journal;
        let other = if multiplier == 1 { 10 } else { 1 };
        bad.body.header.initial_settings = weighted(other);
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }
}

#[test]
fn replay33_rejects_cross_format_generation_pit_and_gameplay_identities() {
    let (_, journal) = recorder(None, 5).into_parts();
    for mutate in 0..8 {
        let mut bad = journal.clone();
        match mutate {
            0 => bad.header.format_version = 32,
            1 => bad.header.generator_version = 6,
            2 => bad.header.generator_version = 5,
            3 => bad.header.spike_pit_layout_version = 1,
            4 => bad.header.gameplay_version = 20,
            5 => bad.header.rules_version = 9,
            6 => bad.header.mob_navigation_version = 2,
            7 => bad.header.punch_activation_version = 1,
            _ => unreachable!(),
        }
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }
}

#[test]
fn replay33_closes_cpu_family_and_difficulty_pairing() {
    for (difficulty, policy) in [
        (CpuDifficulty::Easy, 62),
        (CpuDifficulty::Standard, 62),
        (CpuDifficulty::Hard, 64),
    ] {
        let mut recording = recorder(Some(difficulty), 5);
        ready_humans(&mut recording, 1, 1);
        let expected = recording.runtime().replay_inspection();
        let (_, journal) = recording.into_parts();
        assert_eq!(journal.header.cpu_policy_version, policy);
        verify_exact(&journal, &expected).unwrap();

        let mut old_family = journal.clone();
        let old_policy = if difficulty == CpuDifficulty::Hard {
            61
        } else {
            60
        };
        old_family.header.cpu_policy_version = old_policy;
        old_family.body.header.cpu_policy_version = old_policy;
        old_family.header.cpus[0].policy_version = old_policy;
        old_family.body.header.cpus[0].policy_version = old_policy;
        assert_eq!(replay(&old_family), Err(ReplayError::InvalidHeader));

        let mut forged_difficulty = journal;
        let other = if difficulty == CpuDifficulty::Hard {
            CpuDifficulty::Standard
        } else {
            CpuDifficulty::Hard
        };
        forged_difficulty.header.cpus[0].difficulty = other;
        forged_difficulty.body.header.cpus[0].difficulty = other;
        assert_eq!(replay(&forged_difficulty), Err(ReplayError::InvalidHeader));
    }
}
