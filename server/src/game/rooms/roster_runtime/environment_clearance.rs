use std::sync::Arc;

use super::{
    environment_spike::SpikeRuntimeEnvironment,
    error::RosterRuntimeError,
    runtime::{RosterRuntime, SimulationSeedDerivation},
    types::RosterIdentity,
};
use crate::round_seed::RoundSeedSource;
use grenadiers_core::{game::ArenaConstructionProfile, room::RoomSettings};

impl RosterRuntime {
    /// Diagnostic construction keeps the unweighted prototype reproducible.
    #[cfg(test)]
    pub(in crate::game::rooms) fn new_spike_v20_clearance_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        environment: SpikeRuntimeEnvironment,
        seeds: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_spike_v20_with_navigation(
            identity,
            settings,
            environment.cloud,
            environment.lairs,
            environment.spike_pits,
            seeds,
            simulation_seed,
            environment.navigation,
        )?;
        runtime.construction_profile = ArenaConstructionProfile::LairClearanceV5;
        runtime.cpu_family = super::cpu_family::CpuPolicyFamily::Clearance;
        Ok(runtime)
    }

    pub(in crate::game::rooms) fn new_spike_v21_clearance_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        environment: SpikeRuntimeEnvironment,
        seeds: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_spike_v21_with_navigation(
            identity,
            settings,
            environment,
            seeds,
            simulation_seed,
        )?;
        runtime.construction_profile = ArenaConstructionProfile::WeightedLairClearanceV7;
        runtime.cpu_family = super::cpu_family::CpuPolicyFamily::Clearance;
        Ok(runtime)
    }

    pub(in crate::game::rooms) fn new_spike_v21_reactive_with_navigation(
        identity: RosterIdentity,
        settings: RoomSettings,
        environment: SpikeRuntimeEnvironment,
        seeds: RoundSeedSource,
        simulation_seed: Arc<SimulationSeedDerivation>,
    ) -> Result<Self, RosterRuntimeError> {
        let mut runtime = Self::new_spike_v21_clearance_with_navigation(
            identity,
            settings,
            environment,
            seeds,
            simulation_seed,
        )?;
        runtime.cpu_family = super::cpu_family::CpuPolicyFamily::ReactiveHardV65;
        Ok(runtime)
    }
}
