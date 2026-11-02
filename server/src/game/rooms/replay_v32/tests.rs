use grenadiers_core::{
    game::{EnabledItems, PunchActivationPolicy, SeatId, StormCloudConfig},
    room::{
        BoardSize, BombCapacityDropMultiplier, LairRoomConfiguration, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SpikePitSettings, SpikePitTuning, SpikeRoomConfiguration,
    },
};

use super::*;
use crate::round_seed::RoundSeedSource;

fn configuration() -> SpikeRoomConfiguration {
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::human_duel(),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT.as_legacy(),
        ),
        SpikePitTuning::new(true, SpikePitSettings::new(36, 180).unwrap()),
    )
}

fn weighted(value: u64) -> RoomSettings {
    RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::new(value).unwrap())
}

fn recorder(value: u64) -> RecordingRosterRuntime {
    RecordingRosterRuntime::new(
        configuration(),
        weighted(value),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(32);
            Ok(())
        }),
        Vec::new(),
    )
    .unwrap()
}

fn admit_and_start(recording: &mut RecordingRosterRuntime, settings_revision: u32) {
    for seat in SeatId::DUEL {
        recording.admit(seat, false).unwrap();
        recording
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
        recording.settle([None; 4]).unwrap();
    }
}

#[test]
fn replay32_reconstructs_weighted_initial_and_changed_settings_exactly() {
    let mut recording = recorder(5);
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
    let simulation = expected.simulation.as_ref().unwrap();
    assert_eq!(simulation.epoch, 2);
    assert_eq!(
        simulation.generation.version(),
        grenadiers_core::game::WEIGHTED_GENERATOR_VERSION
    );
    assert_eq!(
        expected
            .lifecycle
            .settings()
            .bomb_capacity_drop_multiplier(),
        BombCapacityDropMultiplier::new(10).unwrap()
    );
    assert_eq!(expected.punch_activation, PunchActivationPolicy::ContactV2);
    let (_, journal) = recording.into_parts();
    assert_eq!(journal.header.format_version, 32);
    assert_eq!(journal.header.gameplay_version, 21);
    assert_eq!(journal.header.generator_version, 6);
    verify_exact(&journal, &expected).unwrap();
}

#[test]
fn replay32_accepts_weight_extrema_and_rejects_mixed_identity() {
    for value in [1, 10] {
        let mut recording = recorder(value);
        admit_and_start(&mut recording, 1);
        let expected = recording.runtime().replay_inspection();
        let (_, journal) = recording.into_parts();
        verify_exact(&journal, &expected).unwrap();

        let mut bad = journal.clone();
        bad.header.gameplay_version = 20;
        assert_eq!(replay(&bad), Err(ReplayError::InvalidHeader));
    }
}

#[test]
fn replay32_binds_weighted_generation_on_every_board() {
    for board in [
        BoardSize::ElevenByNine,
        BoardSize::ThirteenByEleven,
        BoardSize::FifteenByThirteen,
    ] {
        let mut config = configuration();
        let base =
            RoomConfiguration::new(&[grenadiers_core::room::SeatControl::Human; 2], None, board)
                .unwrap();
        config = SpikeRoomConfiguration::new(
            LairRoomConfiguration::new(base, config.base().storm_cloud(), config.base().lairs()),
            config.spike_pits(),
        );
        let mut recording = RecordingRosterRuntime::new(
            config,
            weighted(5),
            RoundSeedSource::with_entropy(move |bytes| {
                bytes.fill(board.width() as u8);
                Ok(())
            }),
            Vec::new(),
        )
        .unwrap();
        admit_and_start(&mut recording, 1);
        let expected = recording.runtime().replay_inspection();
        assert_eq!(
            expected.simulation.as_ref().unwrap().generation.version(),
            grenadiers_core::game::WEIGHTED_GENERATOR_VERSION
        );
        let (_, journal) = recording.into_parts();
        verify_exact(&journal, &expected).unwrap();
    }
}
