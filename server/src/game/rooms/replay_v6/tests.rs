use grenadiers_core::{
    game::{SeatId, StormCloudConfig},
    room::{BoardSize, EnvironmentRoomConfiguration, RoomConfiguration, RoomSettings, SeatControl},
};

use crate::round_seed::RoundSeedSource;

use super::{RecordingRosterRuntime, ReplayAction, replay, verify_exact};

fn recorder(
    board: BoardSize,
    environment: StormCloudConfig,
    count: usize,
) -> RecordingRosterRuntime {
    let base = RoomConfiguration::new(&vec![SeatControl::Human; count], None, board).unwrap();
    RecordingRosterRuntime::new(
        EnvironmentRoomConfiguration::new(base, environment),
        RoomSettings::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(37);
            Ok(())
        }),
        vec![],
    )
    .unwrap()
}

fn ready_and_start(recorder: &mut RecordingRosterRuntime, count: usize) {
    for &seat in &SeatId::ALL[..count] {
        recorder.admit(seat, false).unwrap();
        recorder
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
}

#[test]
fn environment_runtime_constructs_every_board_count_and_disabled_is_still_rules_seven() {
    for board in [
        BoardSize::FifteenByThirteen,
        BoardSize::ThirteenByEleven,
        BoardSize::ElevenByNine,
    ] {
        for count in 2..=4 {
            for environment in [StormCloudConfig::Disabled, StormCloudConfig::Single] {
                let mut recorder = recorder(board, environment, count);
                ready_and_start(&mut recorder, count);
                let inspection = recorder.runtime().replay_inspection();
                let state = &inspection.simulation.unwrap().state;
                assert_eq!(state.rules().version(), 7);
                assert_eq!(state.rules().storm_cloud_config(), environment);
                assert_eq!(inspection.environment, Some(environment));
            }
        }
    }
}

#[test]
fn cloud_only_settings_history_is_recorded_and_replayed_exactly() {
    let mut recorder = recorder(BoardSize::FifteenByThirteen, StormCloudConfig::Single, 2);
    recorder.admit(SeatId::One, false).unwrap();
    recorder.admit(SeatId::Two, false).unwrap();
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT,
                environment: StormCloudConfig::Disabled,
            },
        )
        .unwrap();
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSettings {
                settings_revision: 2,
                settings: RoomSettings::DEFAULT,
                environment: StormCloudConfig::Single,
            },
        )
        .unwrap();
    for seat in [SeatId::One, SeatId::Two] {
        recorder
            .human_action(
                seat,
                1,
                ReplayAction::Ready {
                    ready: true,
                    settings_revision: 3,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 150 {
        recorder.settle([None; 4]).unwrap();
    }
    let inspection = recorder.runtime().replay_inspection();
    assert!(
        inspection
            .simulation
            .as_ref()
            .unwrap()
            .state
            .storm_cloud()
            .is_some()
    );
    let (runtime, journal) = recorder.into_parts();
    verify_exact(&journal, &runtime.replay_inspection()).unwrap();
    assert_eq!(replay(&journal).unwrap(), inspection);
}

#[test]
fn stale_settings_first_start_lock_and_mixed_headers_are_rejected() {
    let mut recorder = recorder(BoardSize::ThirteenByEleven, StormCloudConfig::Single, 2);
    recorder.admit(SeatId::One, false).unwrap();
    recorder.admit(SeatId::Two, false).unwrap();
    assert!(
        recorder
            .human_action(
                SeatId::One,
                1,
                ReplayAction::UpdateSettings {
                    settings_revision: 2,
                    settings: RoomSettings::DEFAULT,
                    environment: StormCloudConfig::Disabled,
                }
            )
            .is_err()
    );
    for seat in [SeatId::One, SeatId::Two] {
        recorder
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    assert!(
        recorder
            .human_action(
                SeatId::One,
                1,
                ReplayAction::UpdateSettings {
                    settings_revision: 1,
                    settings: RoomSettings::DEFAULT,
                    environment: StormCloudConfig::Disabled,
                }
            )
            .is_err()
    );
    let (_, journal) = recorder.into_parts();
    let mut malformed = journal.clone();
    malformed.header.rules_version = 6;
    assert!(replay(&malformed).is_err());
    malformed = journal.clone();
    malformed.header.cpu_policy_version = 17;
    assert!(replay(&malformed).is_err());
    malformed = journal;
    malformed.header.environment_randomness_version = 2;
    assert!(replay(&malformed).is_err());
}

#[test]
fn cloud_only_change_cancels_countdown_and_clears_readiness() {
    let mut recorder = recorder(BoardSize::ElevenByNine, StormCloudConfig::Single, 2);
    for seat in [SeatId::One, SeatId::Two] {
        recorder.admit(seat, false).unwrap();
        recorder
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
    recorder
        .human_action(
            SeatId::One,
            1,
            ReplayAction::UpdateSettings {
                settings_revision: 1,
                settings: RoomSettings::DEFAULT,
                environment: StormCloudConfig::Disabled,
            },
        )
        .unwrap();
    let lifecycle = &recorder.runtime().replay_inspection().lifecycle;
    assert!(matches!(
        lifecycle.phase(),
        grenadiers_core::room::MatchPhase::Waiting
    ));
    assert!(!lifecycle.ready(SeatId::One).unwrap());
    assert!(!lifecycle.ready(SeatId::Two).unwrap());
    let (runtime, journal) = recorder.into_parts();
    verify_exact(&journal, &runtime.replay_inspection()).unwrap();
}

#[test]
fn interruption_and_rematch_retain_environment_and_replay_exactly() {
    let mut recorder = recorder(BoardSize::ThirteenByEleven, StormCloudConfig::Disabled, 2);
    ready_and_start(&mut recorder, 2);
    recorder
        .interrupt(grenadiers_core::room::InterruptReason::ServerStopped)
        .unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        recorder.settle([None; 4]).unwrap();
    }
    for seat in [SeatId::One, SeatId::Two] {
        recorder
            .human_action(seat, 1, ReplayAction::Rematch(true))
            .unwrap();
    }
    let inspection = recorder.runtime().replay_inspection();
    assert_eq!(inspection.environment, Some(StormCloudConfig::Disabled));
    assert!(matches!(
        inspection.lifecycle.phase(),
        grenadiers_core::room::MatchPhase::Waiting
    ));
    let (runtime, journal) = recorder.into_parts();
    verify_exact(&journal, &runtime.replay_inspection()).unwrap();
}
