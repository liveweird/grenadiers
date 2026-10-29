use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SimulationSeed, SpeedLevel, StormCloudConfig},
    room::{DepartureReason, MatchPhase, RoomSettings, RosterParticipant},
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError};
use crate::{
    game::rooms::roster_runtime::tests::{configuration, identity},
    round_seed::RoundSeedSource,
};

fn runtime() -> RosterRuntime {
    runtime_with_settings(RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL))
}

fn runtime_with_settings(settings: RoomSettings) -> RosterRuntime {
    RosterRuntime::new_environment(
        identity(configuration(2)),
        settings,
        StormCloudConfig::Single,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(19);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

fn admit_and_ready(runtime: &mut RosterRuntime, settings_revision: u32) {
    for seat in SeatId::DUEL {
        if runtime.lifecycle.participant(seat).unwrap()
            == grenadiers_core::room::RosterParticipant::NeverAdmitted
        {
            let prepared = runtime.prepare_join(seat).unwrap();
            runtime.commit_admission(prepared);
        }
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision,
                },
            )
            .unwrap();
    }
}

#[test]
fn environment_only_mixed_noop_and_stale_updates_are_atomic() {
    let mut runtime = runtime();
    admit_and_ready(&mut runtime, 1);
    assert!(matches!(
        runtime.lifecycle.phase(),
        MatchPhase::Countdown { .. }
    ));

    let original = runtime.lifecycle.settings();
    let before_environment_change = runtime.replay_inspection();
    let notice = runtime
        .update_environment_settings(SeatId::One, 1, 1, original, StormCloudConfig::Disabled)
        .unwrap()
        .unwrap();
    assert_eq!(runtime.settings_revision, 2);
    assert_eq!(runtime.revision, before_environment_change.revision + 1);
    assert_eq!(runtime.environment(), Some(StormCloudConfig::Disabled));
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    assert!(!runtime.lifecycle.ready(SeatId::One).unwrap());
    assert!(!runtime.lifecycle.ready(SeatId::Two).unwrap());
    assert!(
        !notice
            .environment_room()
            .unwrap()
            .settings
            .environment
            .storm_cloud
    );

    let before_noop = runtime.replay_inspection();
    assert!(
        runtime
            .update_environment_settings(SeatId::One, 2, 2, original, StormCloudConfig::Disabled,)
            .unwrap()
            .is_some()
    );
    assert_eq!(runtime.settings_revision, 2);
    assert_eq!(runtime.lifecycle.settings(), original);
    assert_eq!(runtime.replay_inspection(), before_noop);

    let before_stale = runtime.replay_inspection();
    assert_eq!(
        runtime.update_environment_settings(SeatId::One, 2, 1, original, StormCloudConfig::Single,),
        Err(RosterRuntimeError::StaleSettings)
    );
    assert_eq!(runtime.replay_inspection(), before_stale);

    let mixed = RoomSettings::new(SpeedLevel::Two, 64, 2, 90)
        .unwrap()
        .with_enabled_items(EnabledItems::KICK);
    let before_mixed = runtime.replay_inspection();
    runtime
        .update_environment_settings(SeatId::One, 2, 2, mixed, StormCloudConfig::Single)
        .unwrap();
    assert_eq!(runtime.settings_revision, 3);
    assert_eq!(runtime.revision, before_mixed.revision + 1);
    assert_eq!(runtime.lifecycle.settings(), mixed);
    assert_eq!(runtime.environment(), Some(StormCloudConfig::Single));
}

#[test]
fn disconnected_initial_editor_is_designated_but_cannot_act() {
    let mut runtime = runtime();
    let creator = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(creator);
    let notice = runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap()
        .unwrap();
    assert_eq!(
        notice.environment_room().unwrap().settings_editor,
        Some(grenadiers_core::protocol::v15::Seat::One)
    );
    assert_eq!(
        runtime.update_environment_settings(
            SeatId::One,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
        ),
        Err(RosterRuntimeError::VacantSeat)
    );
}

#[test]
fn forming_environment_departure_reopens_seat_and_preserves_settings() {
    let mut runtime = runtime();
    for seat in SeatId::DUEL {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
    let settings = runtime.lifecycle.settings();
    let settings_revision = runtime.settings_revision;
    admit_and_ready(&mut runtime, settings_revision);
    let revision = runtime.revision;
    let notice = runtime
        .settle_boundary([None, Some(DepartureReason::Explicit), None, None])
        .unwrap()
        .unwrap();
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    assert_eq!(runtime.lifecycle.settings(), settings);
    assert_eq!(runtime.settings_revision, settings_revision);
    assert_eq!(runtime.revision, revision + 1);
    assert_eq!(notice.environment_room().unwrap().round_id.get(), 2);
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Two),
        Ok(RosterParticipant::NeverAdmitted)
    );
    assert!(!runtime.lifecycle.ready(SeatId::One).unwrap());
    assert!(!notice.environment_room().unwrap().seats[1].occupied);

    let replacement = runtime.prepare_join(SeatId::Two).unwrap();
    assert!(replacement.environment_room().unwrap().seats[1].occupied);
    assert_eq!(replacement.environment_room().unwrap().round_id.get(), 2);
    runtime.commit_admission(replacement);
}

#[test]
fn forming_creator_replacement_is_playable_but_settings_authority_stays_revoked() {
    let mut runtime = runtime();
    for seat in SeatId::DUEL {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
    runtime
        .settle_boundary([Some(DepartureReason::Explicit), None, None, None])
        .unwrap();
    assert_eq!(
        runtime.lifecycle.participant(SeatId::One),
        Ok(RosterParticipant::NeverAdmitted)
    );
    let replacement = runtime.prepare_join(SeatId::One).unwrap();
    assert_eq!(
        replacement.environment_room().unwrap().settings_editor,
        None
    );
    runtime.commit_admission(replacement);
    assert_eq!(
        runtime.update_environment_settings(
            SeatId::One,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
    assert!(
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .is_ok()
    );
}

#[test]
fn expired_forming_creator_can_be_replaced_without_restoring_settings_authority() {
    let mut runtime = runtime();
    for seat in SeatId::DUEL {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
    runtime
        .settle_boundary([Some(DepartureReason::TransportLoss), None, None, None])
        .unwrap();
    for _ in 1..grenadiers_core::room::reconnect::RECONNECT_GRACE_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(
        runtime.lifecycle.participant(SeatId::One),
        Ok(RosterParticipant::NeverAdmitted)
    );
    let replacement = runtime.prepare_join(SeatId::One).unwrap();
    assert_eq!(
        replacement.environment_room().unwrap().settings_editor,
        None
    );
}

#[test]
fn completed_match_rematch_does_not_reopen_departed_environment_seats() {
    let mut runtime = runtime_with_settings(
        RoomSettings::new(SpeedLevel::Base, 256, 1, 180)
            .unwrap()
            .with_enabled_items(EnabledItems::ALL),
    );
    admit_and_ready(&mut runtime, 1);
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    runtime
        .lifecycle
        .complete_current_tick(grenadiers_core::game::RoundResult::Winner(SeatId::One))
        .unwrap();
    runtime.reset_for_rematch();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        runtime.lifecycle.advance_idle_tick().unwrap();
    }
    runtime
        .action(SeatId::One, 1, RosterAction::Rematch(true))
        .unwrap();
    runtime
        .action(SeatId::Two, 1, RosterAction::Rematch(true))
        .unwrap();
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Waiting);
    runtime
        .settle_boundary([None, Some(DepartureReason::Explicit), None, None])
        .unwrap();
    assert_eq!(
        runtime.lifecycle.participant(SeatId::Two),
        Ok(RosterParticipant::Departed)
    );
    assert!(runtime.prepare_join(SeatId::Two).is_err());
}

#[test]
fn settings_lock_at_first_start_survives_environment_changes_and_rematches() {
    let mut runtime = runtime();
    let initial = runtime.prepare_join(SeatId::One).unwrap();
    assert_eq!(
        initial.environment_room().unwrap().settings_editor,
        Some(grenadiers_core::protocol::v15::Seat::One)
    );
    runtime.commit_admission(initial);
    admit_and_ready(&mut runtime, 1);
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Running);
    assert!(runtime.settings_locked);
    assert_eq!(
        runtime
            .publish_current()
            .unwrap()
            .environment_room()
            .unwrap()
            .settings_editor,
        None
    );
    assert_eq!(
        runtime.update_environment_settings(
            SeatId::One,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
    runtime.reset_for_rematch();
    assert!(runtime.settings_locked);
    assert_eq!(
        runtime
            .publish_view()
            .unwrap()
            .environment_room()
            .unwrap()
            .settings_editor,
        None
    );
    assert_eq!(
        runtime
            .prepare_resume(SeatId::One)
            .unwrap()
            .environment_room()
            .unwrap()
            .settings_editor,
        None
    );
    assert_eq!(
        runtime.update_environment_settings(
            SeatId::One,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
}
