use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SimulationSeed, StormCloudConfig},
    protocol::{Counter, v14::ResumeToken},
    room::{
        BoardSize, CpuDifficulty, RespawnLairSettings, RespawnLairTuning, RoomConfiguration,
        RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
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

fn runtime_v19(configuration: RoomConfiguration) -> RosterRuntime {
    runtime_for_publication(configuration, true)
}

fn runtime_v20(configuration: RoomConfiguration) -> RosterRuntime {
    RosterRuntime::new_spike_v20_with_navigation(
        identity(configuration),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        lairs(false, 10),
        SpikePitTuning::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(72);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        grenadiers_core::game::MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap()
}

#[test]
fn gameplay20_is_contact_v2_before_the_first_round_and_publishes_rules10() {
    let configuration = RoomConfiguration::human_duel();
    let mut runtime = runtime_v20(configuration);
    assert!(runtime.uses_gameplay20());
    assert_eq!(
        runtime.replay_inspection().punch_activation,
        grenadiers_core::game::PunchActivationPolicy::ContactV2
    );
    admit(&mut runtime);
    let notice = runtime.current_notice().unwrap();
    assert_eq!(notice.spike_v20_room().unwrap().rules_version.get(), 10);
    assert!(notice.spike_v19_room().is_none());
}

fn runtime_v18(configuration: RoomConfiguration) -> RosterRuntime {
    runtime_for_publication(configuration, false)
}

fn runtime_for_publication(configuration: RoomConfiguration, v19: bool) -> RosterRuntime {
    let constructor = if v19 {
        RosterRuntime::new_spike_v19_with_navigation
    } else {
        RosterRuntime::new_spike_with_navigation
    };
    constructor(
        identity(configuration),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        lairs(false, 10),
        SpikePitTuning::DEFAULT,
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
fn explicit_v19_publication_is_distinct_and_preserves_easy_and_standard() {
    let standard = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut old = RosterRuntime::new_spike_with_navigation(
        identity(standard),
        RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL),
        StormCloudConfig::Single,
        lairs(false, 10),
        SpikePitTuning::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(71);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        grenadiers_core::game::MobNavigationPolicy::EscapeV2,
    )
    .unwrap();
    assert!(!old.uses_gameplay19());
    let old_notice = old.current_notice().unwrap();
    assert!(old_notice.spike_room().is_some());
    assert!(old_notice.spike_v19_room().is_none());
    for difficulty in [CpuDifficulty::Easy, CpuDifficulty::Standard] {
        let configuration = RoomConfiguration::new(
            &[SeatControl::Human, SeatControl::Cpu],
            Some(difficulty),
            BoardSize::FifteenByThirteen,
        )
        .unwrap();
        let mut current = runtime_v19(configuration);
        assert!(current.uses_gameplay19());
        let admission = current.prepare_join(SeatId::One).unwrap();
        assert_eq!(admission.spike_v19_room().unwrap().rules_version.get(), 10);
        current.commit_admission(admission);
        let current_notice = current.current_notice().unwrap();
        assert!(current_notice.spike_room().is_none());
        let current_room = current_notice.spike_v19_room().unwrap();
        assert_eq!(
            current_room.configuration.domain().base().base(),
            &configuration
        );
        assert_eq!(current_room.rules_version.get(), 10);
    }
}

#[test]
fn hard_cpu_seats_publish_directly_as_v19_hard() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Cpu],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = runtime_v19(configuration);
    let notice = runtime.current_notice().unwrap();
    assert!(notice.spike_room().is_none());
    let room = notice.spike_v19_room().unwrap();
    assert!(room.seats.iter().skip(1).all(|seat| matches!(
        seat.controller,
        grenadiers_core::protocol::v19::Controller::Cpu {
            difficulty: grenadiers_core::protocol::v19::Difficulty::Hard
        }
    )));
}

#[test]
fn v18_and_v19_standard_publish_equivalent_cpu_observations() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut old = runtime_v18(configuration);
    let mut current = runtime_v19(configuration);
    for runtime in [&mut old, &mut current] {
        let prepared = runtime.prepare_join(SeatId::One).unwrap();
        runtime.commit_admission(prepared);
        runtime
            .action(
                SeatId::One,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
        for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 3 {
            runtime.settle_boundary([None; 4]).unwrap();
        }
    }
    let old_notice = old.current_notice().unwrap();
    let current_notice = current.current_notice().unwrap();
    let world = grenadiers_core::cpu::WorldId {
        room_instance: 7,
        physical_round: old.lifecycle.round_attempt(),
    };
    let old_observation = grenadiers_core::protocol::v18::observe_cpu_recipient(
        old_notice.spike_cpu_publication().unwrap(),
        grenadiers_core::protocol::v18::Seat::Two,
        world,
    )
    .unwrap();
    let current_observation = grenadiers_core::protocol::v19::observe_cpu_recipient(
        current_notice.spike_v19_cpu_publication().unwrap(),
        grenadiers_core::protocol::v19::Seat::Two,
        world,
    )
    .unwrap();
    assert_eq!(old_observation, current_observation);
}

#[test]
fn v19_identity_survives_settings_countdown_snapshot_reconnect_and_interruption() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Easy),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = runtime_v19(configuration);
    let creator = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(creator);
    let settings = runtime.lifecycle.settings();
    let notice = runtime
        .update_spike_settings(
            SeatId::One,
            1,
            1,
            settings,
            StormCloudConfig::Single,
            lairs(false, 10),
            SpikePitTuning::DEFAULT,
        )
        .unwrap()
        .unwrap();
    assert!(notice.spike_room().is_none());
    assert!(notice.spike_v19_room().is_some());

    let countdown = runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap()
        .unwrap();
    assert!(countdown.spike_v19_room().is_some());
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let snapshot = runtime.current_notice().unwrap();
    assert!(snapshot.spike_v19_cpu_publication().is_some());
    assert!(snapshot.frame(SeatId::One).is_some());

    let reconnect = runtime.prepare_resume(SeatId::One).unwrap();
    assert!(reconnect.spike_v19_room().is_some());
    runtime.commit_admission(reconnect);
    let interrupted = runtime
        .interrupt(grenadiers_core::room::InterruptReason::InternalError)
        .unwrap()
        .unwrap();
    assert!(interrupted.spike_v19_room().is_some());
    assert!(interrupted.spike_room().is_none());
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
