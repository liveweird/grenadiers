use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, SeatId, SimulationSeed, StormCloudConfig},
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuDifficulty, RespawnLairTuning, RoomConfiguration,
        RoomSettings, SeatControl, SpikePitTuning,
    },
};

use super::{RosterAction, RosterRuntime, RosterRuntimeError, SpikeRuntimeEnvironment};
use crate::{game::rooms::roster_runtime::tests::identity, round_seed::RoundSeedSource};

fn settings(value: u64) -> RoomSettings {
    RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::new(value).unwrap())
}

fn runtime(configuration: RoomConfiguration, value: u64) -> RosterRuntime {
    RosterRuntime::new_spike_v21_with_navigation(
        identity(configuration),
        settings(value),
        SpikeRuntimeEnvironment {
            cloud: StormCloudConfig::Single,
            lairs: RespawnLairTuning::DEFAULT,
            spike_pits: SpikePitTuning::DEFAULT,
            navigation: grenadiers_core::game::MobNavigationPolicy::TimedEscapeV3,
        },
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(86);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

fn admit(runtime: &mut RosterRuntime, seats: impl IntoIterator<Item = SeatId>) {
    for seat in seats {
        let prepared = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(prepared);
    }
}

#[test]
fn gameplay21_publishes_and_observes_the_accepted_multiplier() {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Standard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = runtime(configuration, 5);
    assert!(runtime.uses_gameplay21());
    admit(&mut runtime, [SeatId::One]);
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
    let notice = runtime.current_notice().unwrap();
    let room = notice.spike_v21_room().unwrap();
    assert_eq!(room.settings.bomb_capacity_drop_multiplier().value(), 5);
    let observation = grenadiers_core::protocol::v21::observe_cpu_recipient(
        notice.spike_v21_cpu_publication().unwrap(),
        grenadiers_core::protocol::v21::Seat::Two,
        grenadiers_core::cpu::WorldId {
            room_instance: 7,
            physical_round: runtime.lifecycle.round_attempt(),
        },
    )
    .unwrap();
    assert_eq!(
        observation
            .parts()
            .rules
            .settings
            .bomb_capacity_drop_multiplier()
            .value(),
        5
    );
    assert_eq!(
        runtime
            .replay_inspection()
            .simulation
            .unwrap()
            .generation
            .version(),
        grenadiers_core::game::WEIGHTED_GENERATOR_VERSION
    );
}

#[test]
fn gameplay21_notices_are_reflexive_and_clone_equal_with_or_without_a_body() {
    let mut runtime = runtime(RoomConfiguration::human_duel(), 5);
    let bodyless = runtime.current_notice().unwrap();
    assert_eq!(bodyless, bodyless);
    assert_eq!(bodyless, bodyless.clone());

    admit(&mut runtime, SeatId::DUEL);
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
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 1 {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let snapshot = runtime.current_notice().unwrap();
    assert!(snapshot.spike_v21_cpu_publication().is_some());
    assert_eq!(snapshot, snapshot);
    assert_eq!(snapshot, snapshot.clone());
}

#[test]
fn multiplier_only_updates_are_atomic_and_noops_do_not_advance_revision() {
    let mut runtime = runtime(RoomConfiguration::human_duel(), 5);
    admit(&mut runtime, SeatId::DUEL);
    runtime
        .action(
            SeatId::Two,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    let notice = runtime
        .update_spike_settings(
            SeatId::One,
            1,
            1,
            settings(10),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT,
            SpikePitTuning::DEFAULT,
        )
        .unwrap()
        .unwrap();
    assert_eq!(runtime.replay_inspection().settings_revision, 2);
    assert!(!runtime.lifecycle.ready(SeatId::Two).unwrap());
    assert_eq!(
        notice
            .spike_v21_room()
            .unwrap()
            .settings
            .bomb_capacity_drop_multiplier()
            .value(),
        10
    );
    let no_op = runtime
        .update_spike_settings(
            SeatId::One,
            1,
            2,
            settings(10),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT,
            SpikePitTuning::DEFAULT,
        )
        .unwrap()
        .unwrap();
    assert_eq!(no_op.spike_v21_room().unwrap().settings_revision.get(), 2);
    assert_eq!(runtime.replay_inspection().settings_revision, 2);
}

#[test]
fn rejected_weight_updates_leave_the_runtime_unchanged() {
    let mut runtime = runtime(RoomConfiguration::human_duel(), 5);
    admit(&mut runtime, SeatId::DUEL);
    for (seat, epoch, revision, expected) in [
        (SeatId::Two, 1, 1, RosterRuntimeError::SettingsLocked),
        (SeatId::One, 2, 1, RosterRuntimeError::StaleEpoch),
        (SeatId::One, 1, 2, RosterRuntimeError::StaleSettings),
    ] {
        let before = runtime.replay_inspection();
        assert_eq!(
            runtime.update_spike_settings(
                seat,
                epoch,
                revision,
                settings(10),
                StormCloudConfig::Disabled,
                RespawnLairTuning::DEFAULT,
                SpikePitTuning::DEFAULT,
            ),
            Err(expected)
        );
        assert_eq!(runtime.replay_inspection(), before);
    }
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
    runtime
        .action(
            SeatId::Two,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS + 1 {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let before = runtime.replay_inspection();
    assert_eq!(
        runtime.update_spike_settings(
            SeatId::One,
            1,
            1,
            settings(10),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT,
            SpikePitTuning::DEFAULT,
        ),
        Err(RosterRuntimeError::SettingsLocked)
    );
    assert_eq!(runtime.replay_inspection(), before);
}
