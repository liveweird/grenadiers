use std::sync::Arc;

use grenadiers_core::{
    game::{MobNavigationPolicy, SeatId, StormCloudConfig},
    room::{RespawnLairTuning, RoomSettings, SpikePitTuning},
};

use crate::round_seed::RoundSeedSource;

use super::{
    error::RosterRuntimeError,
    runtime::{EnvironmentProfile, RosterRuntime, SimulationSeedDerivation},
    types::{RosterIdentity, RosterNotice},
};

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
