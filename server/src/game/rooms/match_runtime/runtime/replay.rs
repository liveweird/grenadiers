use grenadiers_core::{
    game::{GenerationRecord, State, StepReport},
    room::MatchLifecycle,
};

use super::MatchRuntime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RuntimeInspection {
    pub(in crate::game::rooms) lifecycle: MatchLifecycle,
    pub(in crate::game::rooms) simulation: Option<SimulationInspection>,
    pub(in crate::game::rooms) inputs: String,
    pub(in crate::game::rooms) reports: Vec<StepReport>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct SimulationInspection {
    pub(in crate::game::rooms) epoch: u64,
    pub(in crate::game::rooms) state: State,
    pub(in crate::game::rooms) generation: Option<GenerationRecord>,
}

impl MatchRuntime {
    pub(in crate::game::rooms) fn replay_inspection(&self) -> RuntimeInspection {
        RuntimeInspection {
            lifecycle: self.lifecycle.clone(),
            simulation: self.sim.as_ref().map(|sim| SimulationInspection {
                epoch: sim.epoch,
                state: sim.state.clone(),
                generation: sim.generation_record,
            }),
            inputs: format!("{:?}", self.inputs),
            reports: self.reports.clone(),
        }
    }
}
