use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, v14::ResumeToken},
    room::{
        RespawnLairSettings, RespawnLairTuning, RoomSettings, SpikePitSettings, SpikePitTuning,
    },
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError};
use crate::{
    game::rooms::roster_runtime::tests::{configuration, identity},
    round_seed::RoundSeedSource,
};

fn lairs(enabled: bool, seconds: u8) -> RespawnLairTuning {
    RespawnLairTuning::new(enabled, RespawnLairSettings::new(seconds).unwrap())
}

fn pits(enabled: bool, warning: u64, raised: u64) -> SpikePitTuning {
    SpikePitTuning::new(enabled, SpikePitSettings::new(warning, raised).unwrap())
}

fn runtime(spike_pits: SpikePitTuning) -> RosterRuntime {
    RosterRuntime::new_spike_with_navigation(
        identity(configuration(2)),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        lairs(false, 10),
        spike_pits,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(71);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        grenadiers_core::game::MobNavigationPolicy::EscapeV2,
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
fn spike_defaults_publish_v18_rules10_and_recipient_checked_admission() {
    let mut runtime = runtime(SpikePitTuning::DEFAULT);
    admit(&mut runtime);
    let room = runtime.current_notice().unwrap();
    let room = room.spike_room().unwrap();
    assert_eq!(room.rules_version.get(), 10);
    assert_eq!(room.settings.environment.spike_pits.warning_ticks, 36);
    assert_eq!(room.settings.environment.spike_pits.raised_ticks, 180);
    assert!(room.settings.environment.spike_pits.enabled);

    let prepared = runtime.prepare_resume(SeatId::One).unwrap();
    assert_eq!(prepared.spike_room().unwrap().rules_version.get(), 10);
    let encoded = prepared
        .encode(
            Counter::new(7).unwrap(),
            ResumeToken::new("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
                .unwrap(),
        )
        .unwrap();
    assert!(encoded.contains("\"rulesVersion\":10"));
}

#[test]
fn only_creator_can_change_spike_settings_and_disabled_pits_retain_durations() {
    let mut runtime = runtime(SpikePitTuning::DEFAULT);
    admit(&mut runtime);
    let retained = pits(false, 12, 240);
    assert_eq!(
        runtime.update_spike_settings(
            SeatId::Two,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
            lairs(true, 6),
            retained,
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
    let notice = runtime
        .update_spike_settings(
            SeatId::One,
            1,
            1,
            runtime.lifecycle.settings(),
            StormCloudConfig::Disabled,
            lairs(true, 6),
            retained,
        )
        .unwrap()
        .unwrap();
    assert_eq!(runtime.spike_tuning(), Some(retained));
    let environment = notice.spike_room().unwrap().settings.environment;
    assert!(!environment.spike_pits.enabled);
    assert_eq!(environment.spike_pits.warning_ticks, 12);
    assert_eq!(environment.spike_pits.raised_ticks, 240);

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
    let rules = runtime.sim.as_ref().unwrap().state.rules();
    assert_eq!(rules.version(), 10);
    assert_eq!(
        rules.spike_pit_config(),
        grenadiers_core::room::SpikePitConfig::Disabled
    );
}
