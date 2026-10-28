use grenadiers_core::{
    game::{SeatId, StepReport},
    room::{MatchChange, MatchLifecycle, MatchPhase},
};

use super::{
    InternalWireProfile, MatchNotice, MatchRuntime, MatchRuntimeError, SimRound,
    map_lifecycle_error,
};
use crate::game::rooms::input::InputMailbox;

pub(super) struct LiveStage {
    lifecycle: MatchLifecycle,
    revision: u32,
    settings_revision: u32,
    settings_editor: Option<SeatId>,
    inputs: InputMailbox,
    sim: Option<SimRound>,
    reports: Vec<StepReport>,
}

impl MatchRuntime {
    pub(super) fn live_stage(&self) -> LiveStage {
        LiveStage {
            lifecycle: self.lifecycle.clone(),
            revision: self.revision,
            settings_revision: self.settings_revision,
            settings_editor: self.settings_editor,
            inputs: self.inputs.clone(),
            sim: self.sim.clone(),
            reports: self.reports.clone(),
        }
    }

    pub(super) fn restore_live_stage(&mut self, stage: LiveStage) {
        self.lifecycle = stage.lifecycle;
        self.revision = stage.revision;
        self.settings_revision = stage.settings_revision;
        self.settings_editor = stage.settings_editor;
        self.inputs = stage.inputs;
        self.sim = stage.sim;
        self.reports = stage.reports;
    }

    pub(super) fn initialize_live_cpu_consensus(&mut self) {
        if self.live_solo() {
            self.lifecycle
                .set_ready(SeatId::Two, true)
                .expect("occupied live solo CPU can be ready in the waiting room");
        }
    }

    pub(super) fn settle_live_cpu_consensus(&mut self) -> Result<bool, MatchRuntimeError> {
        if !self.live_solo() {
            return Ok(false);
        }
        let mut changed = false;
        match self.lifecycle.phase() {
            MatchPhase::Waiting if !self.lifecycle.seats()[SeatId::Two.index()].ready() => {
                changed |= self
                    .lifecycle
                    .set_ready(SeatId::Two, true)
                    .map_err(map_lifecycle_error)?
                    != MatchChange::Unchanged;
            }
            MatchPhase::MatchResult { rematch_at, .. }
                if self.lifecycle.tick() >= rematch_at
                    && !self.lifecycle.series().rematch_votes()[SeatId::Two.index()] =>
            {
                let change = self
                    .lifecycle
                    .set_rematch_vote(SeatId::Two, true)
                    .map_err(map_lifecycle_error)?;
                changed |= change != MatchChange::Unchanged;
                if change == MatchChange::RematchStarted {
                    self.clear_round();
                    self.lifecycle
                        .set_ready(SeatId::Two, true)
                        .map_err(map_lifecycle_error)?;
                }
            }
            _ => {}
        }
        Ok(changed)
    }

    pub(super) fn settle_candidate_cpu(
        &self,
        lifecycle: &mut MatchLifecycle,
    ) -> Result<bool, MatchRuntimeError> {
        if !self.live_solo() {
            return Ok(false);
        }
        let mut clear_round = false;
        match lifecycle.phase() {
            MatchPhase::Waiting if !lifecycle.seats()[SeatId::Two.index()].ready() => {
                lifecycle
                    .set_ready(SeatId::Two, true)
                    .map_err(map_lifecycle_error)?;
            }
            MatchPhase::MatchResult { rematch_at, .. }
                if lifecycle.tick() >= rematch_at
                    && !lifecycle.series().rematch_votes()[SeatId::Two.index()] =>
            {
                let change = lifecycle
                    .set_rematch_vote(SeatId::Two, true)
                    .map_err(map_lifecycle_error)?;
                match change {
                    MatchChange::RematchStarted => {
                        clear_round = true;
                        lifecycle
                            .set_ready(SeatId::Two, true)
                            .map_err(map_lifecycle_error)?;
                    }
                    MatchChange::Unchanged | MatchChange::Changed => {}
                    MatchChange::CountdownStarted
                    | MatchChange::RoundStarted(_)
                    | MatchChange::RoundFinished
                    | MatchChange::MatchFinished(_) => {
                        return Err(MatchRuntimeError::Publication);
                    }
                }
            }
            _ => {}
        }
        Ok(clear_round)
    }

    pub(super) fn view_notice(
        &self,
        room: grenadiers_core::protocol::v13::RoomView,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        let frames = match self.wire_profile {
            InternalWireProfile::Legacy13 => None,
            InternalWireProfile::Live14 => Some(super::super::SeatFrames::shared(
                crate::game::rooms::legacy_v14::room_state(&room)
                    .map_err(|_| MatchRuntimeError::Publication)?,
            )),
        };
        Ok(MatchNotice {
            room,
            frames,
            cpu_observation: None,
        })
    }

    pub(super) fn refresh_live_notice(
        &self,
        notice: MatchNotice,
    ) -> Result<MatchNotice, MatchRuntimeError> {
        if self.wire_profile != InternalWireProfile::Live14 {
            return Ok(notice);
        }
        let room = super::super::publication::room_view(
            &self.identity,
            &self.lifecycle,
            self.revision,
            self.settings_revision,
            self.settings_editor,
        )?;
        if self.sim.is_some()
            && matches!(
                self.lifecycle.phase(),
                MatchPhase::Running | MatchPhase::Result { .. } | MatchPhase::MatchResult { .. }
            )
        {
            let sim = self.sim.as_ref().ok_or(MatchRuntimeError::Publication)?;
            let frames = crate::game::rooms::legacy_v14::snapshot(
                &room,
                super::super::publication::wire_counter(sim.epoch)?,
                &sim.state,
                &self.reports,
                sim.generation_record,
            )
            .map_err(|_| MatchRuntimeError::Publication)?;
            Ok(MatchNotice {
                room,
                frames: Some(super::super::SeatFrames::new(frames)),
                cpu_observation: notice.cpu_observation,
            })
        } else {
            self.view_notice(room)
        }
    }

    pub(super) fn is_live14(&self) -> bool {
        self.wire_profile == InternalWireProfile::Live14
    }

    fn live_solo(&self) -> bool {
        self.wire_profile == InternalWireProfile::Live14
            && matches!(self.identity.mode, grenadiers_core::room::RoomMode::Solo(_))
    }
}
