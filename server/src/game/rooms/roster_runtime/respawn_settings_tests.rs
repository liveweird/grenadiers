use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, v14::ResumeToken},
    room::{RespawnLairSettings, RespawnLairTuning, RoomSettings},
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError};
use crate::{
    game::rooms::roster_runtime::tests::{configuration, identity},
    round_seed::RoundSeedSource,
};

fn tuning(enabled: bool, seconds: u8) -> RespawnLairTuning {
    RespawnLairTuning::new(enabled, RespawnLairSettings::new(seconds).unwrap())
}

fn runtime(initial: RespawnLairTuning) -> RosterRuntime {
    RosterRuntime::new_respawn(
        identity(configuration(2)),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        initial,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(51);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

fn admit(runtime: &mut RosterRuntime) {
    for seat in SeatId::DUEL {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
}

#[test]
fn respawn_settings_publish_rules9_and_lock_after_round_start() {
    let mut runtime = runtime(tuning(false, 17));
    admit(&mut runtime);
    let changed = tuning(true, 5);
    let notice = runtime
        .update_respawn_settings(
            SeatId::One,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
            changed,
        )
        .unwrap()
        .unwrap();
    assert_eq!(runtime.respawn_tuning(), Some(changed));
    let room = notice.respawn_room().unwrap();
    assert_eq!(room.rules_version.get(), 9);
    assert_eq!(room.settings.environment.lairs.max_living_mobs, 1);
    assert!(!room.settings.environment.storm_cloud);

    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 2,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(runtime.sim.as_ref().unwrap().state.rules().version(), 9);
    assert_eq!(
        runtime.update_respawn_settings(
            SeatId::One,
            1,
            2,
            runtime.lifecycle.settings(),
            StormCloudConfig::Single,
            tuning(true, 6),
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
}

#[test]
fn respawn_admission_and_snapshots_are_recipient_checked_v17() {
    let mut runtime = runtime(tuning(true, 5));
    admit(&mut runtime);
    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(prepared.respawn_room().unwrap().rules_version.get(), 9);
    let encoded = prepared
        .encode(
            Counter::new(7).unwrap(),
            ResumeToken::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
        )
        .unwrap();
    assert!(encoded.contains("\"rulesVersion\":9"));
    runtime.commit_admission(prepared);
    for _ in 0..3 {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let notice = runtime.current_notice().unwrap();
    assert!(notice.respawn_cpu_publication().is_some());
    assert!(notice.frame(SeatId::One).is_some());
    assert!(notice.frame(SeatId::Two).is_some());
}
