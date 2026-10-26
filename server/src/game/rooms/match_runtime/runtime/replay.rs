use grenadiers_core::{
    game::{GenerationRecord, State, StepReport},
    room::{DepartureReason, MatchLifecycle, MatchPhase, RoomSettings},
};

use super::{MatchRuntime, MatchRuntimeError, map_lifecycle_error};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct RuntimeInspection {
    pub(in crate::game::rooms) lifecycle: MatchLifecycle,
    pub(in crate::game::rooms) simulation: Option<SimulationInspection>,
    pub(in crate::game::rooms) inputs: String,
    pub(in crate::game::rooms) reports: Vec<StepReport>,
    pub(in crate::game::rooms) settings: RoomSettings,
    pub(in crate::game::rooms) settings_revision: u32,
    pub(in crate::game::rooms) settings_editor: Option<grenadiers_core::game::SeatId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) struct SimulationInspection {
    pub(in crate::game::rooms) epoch: u64,
    pub(in crate::game::rooms) state: State,
    pub(in crate::game::rooms) generation: Option<GenerationRecord>,
}

impl MatchRuntime {
    pub(in crate::game::rooms) fn sim(&self) -> Option<&super::SimRound> {
        self.sim.as_ref()
    }

    pub(in crate::game::rooms) fn replace_state_for_test(&mut self, state: State) {
        self.sim.as_mut().expect("running simulation").state = state;
        self.reports.clear();
    }

    pub(in crate::game::rooms) fn set_revision(&mut self, revision: u32) {
        self.revision = revision;
    }

    pub(in crate::game::rooms) fn set_settings_revision(&mut self, revision: u32) {
        self.settings_revision = revision;
    }

    pub(in crate::game::rooms) fn replace_reports_for_test(&mut self, reports: Vec<StepReport>) {
        self.reports = reports;
    }

    pub(in crate::game::rooms) const fn publication_revision_for_test(&self) -> u32 {
        self.revision
    }

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
            settings: self.lifecycle.settings(),
            settings_revision: self.settings_revision,
            settings_editor: self.settings_editor,
        }
    }

    pub(in crate::game::rooms) fn replay_tick_profile(
        &mut self,
        profile: u16,
    ) -> Result<(), MatchRuntimeError> {
        let previous = self.lifecycle.phase();
        self.ensure_revision()?;
        let publish = match previous {
            MatchPhase::Running => self.running_tick()?,
            MatchPhase::Interrupted { .. } => false,
            phase => self.idle_tick(phase)?,
        };
        if profile == crate::game::rooms::replay::M5_LIFECYCLE_PROFILE
            && matches!(previous, MatchPhase::Result { .. })
            && matches!(self.lifecycle.phase(), MatchPhase::Countdown { .. })
        {
            self.clear_round();
        }
        if profile == crate::game::rooms::replay::M5_LIFECYCLE_PROFILE && publish {
            self.publish_current()?;
        }
        Ok(())
    }

    pub(in crate::game::rooms) fn replay_grace_depart(
        &mut self,
        departures: [Option<DepartureReason>; 2],
    ) -> Result<(), MatchRuntimeError> {
        self.apply_grace_departures(departures).map(|_| ())
    }

    pub(in crate::game::rooms) fn replay_expiry(&mut self) -> Result<(), MatchRuntimeError> {
        self.lifecycle
            .expire_reconnect_grace()
            .map_err(map_lifecycle_error)?;
        self.release_expired_editor();
        Ok(())
    }

    pub(in crate::game::rooms) fn replay_report_flush(&mut self) -> Result<(), MatchRuntimeError> {
        self.publish_current().map(|_| ())
    }
}
