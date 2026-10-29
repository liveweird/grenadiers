use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SimulationSeed, SpeedLevel, StormCloudConfig},
    room::{MatchPhase, RoomSettings},
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError};
use crate::{
    game::rooms::roster_runtime::tests::{configuration, identity},
    round_seed::RoundSeedSource,
};

fn runtime() -> RosterRuntime {
    RosterRuntime::new_environment(
        identity(configuration(2)),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
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
fn settings_lock_at_first_start_survives_environment_changes_and_rematches() {
    let mut runtime = runtime();
    admit_and_ready(&mut runtime, 1);
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.lifecycle.phase(), MatchPhase::Running);
    assert!(runtime.settings_locked);
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
