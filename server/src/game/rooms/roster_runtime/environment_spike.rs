use std::sync::Arc;

use grenadiers_core::{
    game::{MobNavigationPolicy, SeatId, StormCloudConfig},
    room::{RespawnLairTuning, RoomSettings, SpikePitTuning},
};

use crate::round_seed::RoundSeedSource;

use super::{
    error::RosterRuntimeError,
    runtime::{
        EnvironmentProfile, RosterRuntime, SimulationSeedDerivation, SpikePublicationVersion,
    },
    types::{RosterIdentity, RosterNotice},
};

#[derive(Clone, Copy)]
pub(in crate::game::rooms) struct SpikeRuntimeEnvironment {
    pub(in crate::game::rooms) cloud: StormCloudConfig,
    pub(in crate::game::rooms) lairs: RespawnLairTuning,
    pub(in crate::game::rooms) spike_pits: SpikePitTuning,
    pub(in crate::game::rooms) navigation: MobNavigationPolicy,
}

impl RosterRuntime {
    pub(in crate::game::rooms) fn new_spike(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        lairs: RespawnLairTuning,
        spike_pits: SpikePitTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        if settings.bomb_capacity_drop_multiplier()
            != grenadiers_core::room::BombCapacityDropMultiplier::LEGACY
        {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        }
        let mut runtime = Self::new(identity, settings, round_seed_source, simulation_seed)?;
        runtime.environment_profile = Some(EnvironmentProfile::SpikeV10 {
            cloud,
            lairs,
            spike_pits,
        });
        Ok(runtime)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::game::rooms) fn new_spike_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        lairs: RespawnLairTuning,
        spike_pits: SpikePitTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
        navigation: MobNavigationPolicy,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_spike(
            identity,
            settings,
            cloud,
            lairs,
            spike_pits,
            round_seed_source,
            simulation_seed,
        )?;
        runtime.mob_navigation = navigation;
        Ok(runtime)
    }

    #[allow(clippy::too_many_arguments)]
    #[cfg_attr(not(test), allow(dead_code))]
    pub(in crate::game::rooms) fn new_spike_v19_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        lairs: RespawnLairTuning,
        spike_pits: SpikePitTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
        navigation: MobNavigationPolicy,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_spike_with_navigation(
            identity,
            settings,
            cloud,
            lairs,
            spike_pits,
            round_seed_source,
            simulation_seed,
            navigation,
        )?;
        runtime.spike_publication = SpikePublicationVersion::V19;
        Ok(runtime)
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::game::rooms) fn new_spike_v20_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        lairs: RespawnLairTuning,
        spike_pits: SpikePitTuning,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
        navigation: MobNavigationPolicy,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_spike_with_navigation(
            identity,
            settings,
            cloud,
            lairs,
            spike_pits,
            round_seed_source,
            simulation_seed,
            navigation,
        )?;
        runtime.spike_publication = SpikePublicationVersion::V20;
        runtime.punch_activation = grenadiers_core::game::PunchActivationPolicy::ContactV2;
        Ok(runtime)
    }

    pub(in crate::game::rooms) fn new_spike_v21_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        environment: SpikeRuntimeEnvironment,
        round_seed_source: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = if settings.bomb_capacity_drop_multiplier()
            == grenadiers_core::room::BombCapacityDropMultiplier::LEGACY
        {
            Self::new_spike_v20_with_navigation(
                identity,
                settings,
                environment.cloud,
                environment.lairs,
                environment.spike_pits,
                round_seed_source,
                simulation_seed,
                environment.navigation,
            )?
        } else {
            let mut runtime = Self::new(identity, settings, round_seed_source, simulation_seed)?;
            runtime.environment_profile = Some(EnvironmentProfile::SpikeV10 {
                cloud: environment.cloud,
                lairs: environment.lairs,
                spike_pits: environment.spike_pits,
            });
            runtime.mob_navigation = environment.navigation;
            runtime
        };
        runtime.spike_publication = SpikePublicationVersion::V21;
        runtime.punch_activation = grenadiers_core::game::PunchActivationPolicy::ContactV2;
        runtime.arena_generation = super::runtime::ArenaGenerationProfile::WeightedV6;
        Ok(runtime)
    }

    pub(in crate::game::rooms) const fn spike_tuning(&self) -> Option<SpikePitTuning> {
        match self.environment_profile {
            Some(EnvironmentProfile::SpikeV10 { spike_pits, .. }) => Some(spike_pits),
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(in crate::game::rooms) fn update_spike_settings(
        &mut self,
        seat: SeatId,
        expected_epoch: u64,
        expected: u32,
        settings: RoomSettings,
        cloud: StormCloudConfig,
        lairs: RespawnLairTuning,
        spike_pits: SpikePitTuning,
    ) -> Result<Option<RosterNotice>, RosterRuntimeError> {
        let mut next = self.clone();
        if !matches!(
            next.environment_profile,
            Some(EnvironmentProfile::SpikeV10 { .. })
        ) {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        }
        if next.arena_generation == super::runtime::ArenaGenerationProfile::LegacyV3V4
            && settings.bomb_capacity_drop_multiplier()
                != grenadiers_core::room::BombCapacityDropMultiplier::LEGACY
        {
            return Err(RosterRuntimeError::UnsupportedConfiguration);
        }
        let notice = next.apply_profile_settings(
            seat,
            expected_epoch,
            expected,
            settings,
            EnvironmentProfile::SpikeV10 {
                cloud,
                lairs,
                spike_pits,
            },
        )?;
        *self = next;
        Ok(notice)
    }
}
