use grenadiers_core::{
    game::{GenerationRecord, State, StepReport, StormCloudConfig},
    room::RosterMatchLifecycle,
};

use super::runtime::RosterRuntime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RosterRuntimeInspection {
    pub(in crate::game::rooms) lifecycle: RosterMatchLifecycle,
    pub(in crate::game::rooms) simulation: Option<RosterSimulationInspection>,
    pub(in crate::game::rooms) inputs: String,
    pub(in crate::game::rooms) reports: Vec<StepReport>,
    pub(in crate::game::rooms) revision: u32,
    pub(in crate::game::rooms) settings_revision: u32,
    pub(in crate::game::rooms) environment: Option<StormCloudConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RosterSimulationInspection {
    pub(in crate::game::rooms) epoch: u64,
    pub(in crate::game::rooms) state: State,
    pub(in crate::game::rooms) generation: GenerationRecord,
}

impl RosterRuntime {
    pub(in crate::game::rooms) fn replay_inspection(&self) -> RosterRuntimeInspection {
        RosterRuntimeInspection {
            lifecycle: self.lifecycle.clone(),
            simulation: self.sim.as_ref().map(|sim| RosterSimulationInspection {
                epoch: sim.epoch,
                state: sim.state.clone(),
                generation: sim.generation,
            }),
            inputs: format!("{:?}", self.inputs),
            reports: self.reports.clone(),
            revision: self.revision,
            settings_revision: self.settings_revision,
            environment: self.environment,
        }
    }
}
