use std::sync::Arc;

use grenadiers_core::{
    game::{EnabledItems, MobNavigationPolicy, SimulationSeed, StormCloudConfig},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuDifficulty, CpuProfile, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitTuning,
    },
};

use crate::{
    game::rooms::roster_runtime::{RosterIdentity, RosterRuntime, SpikeRuntimeEnvironment},
    round_seed::RoundSeedSource,
};

pub(super) fn runtime(profile: CpuProfile, instance: u64) -> (RosterRuntime, RoomSettings) {
    let settings = RoomSettings::new(grenadiers_core::game::SpeedLevel::Four, 256, 3, 240)
        .unwrap()
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT);
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let identity = RosterIdentity {
        id: RoomId::new("0123456789abcdef0123456789abc067").unwrap(),
        code: RoomCode::new("ABCDEFGH").unwrap(),
        registry_serial: instance,
        configuration,
    };
    let environment = SpikeRuntimeEnvironment {
        cloud: StormCloudConfig::Disabled,
        lairs: RespawnLairTuning::DEFAULT,
        spike_pits: SpikePitTuning::DEFAULT,
        navigation: MobNavigationPolicy::TimedEscapeV3,
    };
    let seeds = RoundSeedSource::with_entropy(|bytes| {
        bytes.fill(67);
        Ok(())
    });
    let simulation_seed = Arc::new(|_| Ok(SimulationSeed::new([67; 32])));
    let runtime = match profile {
        CpuProfile::HARD_V67 => RosterRuntime::new_spike_v21_combination_with_navigation,
        CpuProfile::HARD_V69 => RosterRuntime::new_spike_v21_progress_with_navigation,
        CpuProfile::HARD_V70 => RosterRuntime::new_spike_v21_opportunity_with_navigation,
        _ => RosterRuntime::new_spike_v21_pickup_with_navigation,
    }(identity, settings, environment, seeds, simulation_seed)
    .unwrap();
    (runtime, settings)
}
