use std::sync::Arc;

use grenadiers_core::{
    game::{
        ArenaConstructionProfile, EnabledItems, MobNavigationPolicy, SimulationSeed,
        StormCloudConfig,
    },
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuProfile, RespawnLairTuning, RoomSettings,
        SpikePitTuning,
    },
};

use super::{RosterRuntime, SpikeRuntimeEnvironment, cpu_family::CpuPolicyFamily, tests};
use crate::round_seed::RoundSeedSource;

fn runtime(reactive: bool) -> RosterRuntime {
    runtime_with_family(if reactive {
        CpuPolicyFamily::ReactiveHardV65
    } else {
        CpuPolicyFamily::Clearance
    })
}

fn runtime_with_family(family: CpuPolicyFamily) -> RosterRuntime {
    let constructor = match family {
        CpuPolicyFamily::Historical => panic!("use the historical runtime fixture"),
        CpuPolicyFamily::Clearance => RosterRuntime::new_spike_v21_clearance_with_navigation,
        CpuPolicyFamily::ReactiveHardV65 => RosterRuntime::new_spike_v21_reactive_with_navigation,
        CpuPolicyFamily::PickupHardV66 => RosterRuntime::new_spike_v21_pickup_with_navigation,
    };
    constructor(
        tests::identity(tests::configuration_for(2, BoardSize::ThirteenByEleven)),
        RoomSettings::DEFAULT
            .with_enabled_items(EnabledItems::ALL)
            .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT),
        SpikeRuntimeEnvironment {
            cloud: StormCloudConfig::Single,
            lairs: RespawnLairTuning::DEFAULT,
            spike_pits: SpikePitTuning::DEFAULT,
            navigation: MobNavigationPolicy::TimedEscapeV3,
        },
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(65);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
    )
    .unwrap()
}

#[test]
fn planner_family_changes_do_not_reinterpret_the_arena_or_simulation() {
    let mut archived = runtime(false);
    let mut reactive = runtime(true);
    assert_eq!(archived.cpu_family, CpuPolicyFamily::Clearance);
    assert_eq!(reactive.cpu_family, CpuPolicyFamily::ReactiveHardV65);
    assert_eq!(archived.hard_cpu_profile(), Some(CpuProfile::HARD_V64));
    assert_eq!(reactive.hard_cpu_profile(), Some(CpuProfile::HARD_V65));
    assert!(archived.uses_clearance_cpu_profiles());
    assert!(reactive.uses_clearance_cpu_profiles());
    tests::start(&mut archived, 2);
    tests::start(&mut reactive, 2);
    let old = archived.replay_inspection();
    let new = reactive.replay_inspection();
    assert_eq!(
        old.construction_profile,
        ArenaConstructionProfile::WeightedLairClearanceV7
    );
    assert_eq!(old.construction_profile, new.construction_profile);
    assert_eq!(old.simulation, new.simulation);
    assert_eq!(old.lifecycle, new.lifecycle);
    assert_ne!(old.cpu_family, new.cpu_family);
}

#[test]
fn cpu_family_survives_settings_updates_and_simulation_resets() {
    for family in [
        CpuPolicyFamily::ReactiveHardV65,
        CpuPolicyFamily::PickupHardV66,
    ] {
        assert_family_survives_settings_and_resets(family);
    }
}

fn assert_family_survives_settings_and_resets(family: CpuPolicyFamily) {
    use grenadiers_core::game::SeatId;
    let mut runtime = runtime_with_family(family);
    tests::admit_all(&mut runtime, 2);
    let settings = runtime
        .lifecycle
        .settings()
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::new(10).unwrap());
    runtime
        .update_spike_settings(
            SeatId::One,
            1,
            1,
            settings,
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT,
            SpikePitTuning::DEFAULT,
        )
        .unwrap();
    assert_eq!(runtime.cpu_family, family);
    ready_all(&mut runtime);
    runtime
        .interrupt(grenadiers_core::room::InterruptReason::ServerStopped)
        .unwrap();
    for _ in 0..grenadiers_core::room::series::RESULT_HOLD_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    for seat in SeatId::DUEL {
        runtime
            .action(
                seat,
                runtime.lifecycle.round_attempt(),
                super::RosterAction::Rematch(true),
            )
            .unwrap();
    }
    assert_eq!(
        runtime.lifecycle.phase(),
        grenadiers_core::room::MatchPhase::Waiting
    );
    ready_all(&mut runtime);
    assert_eq!(runtime.cpu_family, family);
    assert_eq!(
        runtime
            .lifecycle
            .settings()
            .bomb_capacity_drop_multiplier()
            .value(),
        10
    );
    assert_eq!(
        runtime.construction_profile,
        ArenaConstructionProfile::WeightedLairClearanceV7
    );
}

fn ready_all(runtime: &mut RosterRuntime) {
    for seat in grenadiers_core::game::SeatId::DUEL {
        runtime
            .action(
                seat,
                runtime.lifecycle.round_attempt(),
                super::RosterAction::Ready {
                    ready: true,
                    settings_revision: runtime.settings_revision,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(
        runtime.lifecycle.phase(),
        grenadiers_core::room::MatchPhase::Running
    );
}

#[test]
fn pickup_family_changes_only_the_selected_hard_planner() {
    let mut reactive = runtime(true);
    let mut pickup = runtime_with_family(CpuPolicyFamily::PickupHardV66);
    assert_eq!(pickup.hard_cpu_profile(), Some(CpuProfile::HARD_V66));
    assert!(pickup.uses_clearance_cpu_profiles());
    tests::start(&mut reactive, 2);
    tests::start(&mut pickup, 2);
    let previous = reactive.replay_inspection();
    let current = pickup.replay_inspection();
    assert_eq!(previous.simulation, current.simulation);
    assert_eq!(previous.lifecycle, current.lifecycle);
    assert_eq!(previous.construction_profile, current.construction_profile);
    assert_ne!(previous.cpu_family, current.cpu_family);
    assert_eq!(tests::runtime(2).hard_cpu_profile(), None);
}
