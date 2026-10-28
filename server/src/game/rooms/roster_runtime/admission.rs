use grenadiers_core::{
    game::{GenerationRecord, SeatId},
    protocol::{Counter, v14},
    room::{MatchPhase, RosterMatchLifecycle, RosterParticipant},
};

use super::{
    error::{RosterRuntimeError, lifecycle as lifecycle_error},
    publication,
    runtime::RosterRuntime,
    types::{RosterNotice, wire_seat},
};

pub(in crate::game::rooms) struct PreparedRosterAdmission {
    lifecycle: RosterMatchLifecycle,
    revision: u32,
    seat: SeatId,
    pub(in crate::game::rooms) is_replacement: bool,
    pub(in crate::game::rooms) room: v14::RoomView,
    pub(in crate::game::rooms) state_epoch: Option<Counter>,
    pub(in crate::game::rooms) capture: Option<v14::ExpandedCapture>,
    pub(in crate::game::rooms) generation: Option<GenerationRecord>,
    pub(in crate::game::rooms) last_action_id: Option<Counter>,
    pub(in crate::game::rooms) notice: RosterNotice,
}

impl PreparedRosterAdmission {
    pub(in crate::game::rooms) fn encode(
        &self,
        sequence: Counter,
        resume_token: v14::ResumeToken,
    ) -> Result<String, RosterRuntimeError> {
        v14::prepare_admission(
            v14::AdmissionProjection {
                room: &self.room,
                state_epoch: self.state_epoch,
                capture: self.capture.as_ref(),
                generation: self.generation,
            },
            v14::PersonalAdmission {
                recipient: wire_seat(self.seat),
                last_action_id: self.last_action_id,
                sequence,
                resume_token,
            },
        )
        .map_err(|_| RosterRuntimeError::Publication)
    }
}

impl RosterRuntime {
    pub(in crate::game::rooms) fn prepare_join(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRosterAdmission, RosterRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        lifecycle.occupy(seat).map_err(lifecycle_error)?;
        self.prepare_admission(seat, lifecycle, false)
    }

    pub(in crate::game::rooms) fn prepare_resume(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRosterAdmission, RosterRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        let is_replacement = match lifecycle.participant(seat).map_err(lifecycle_error)? {
            RosterParticipant::Reserved { .. } => {
                lifecycle.resume_connection(seat).map_err(lifecycle_error)?;
                false
            }
            RosterParticipant::Connected => true,
            RosterParticipant::NeverAdmitted
            | RosterParticipant::Departed
            | RosterParticipant::Cpu => return Err(RosterRuntimeError::StaleEpoch),
        };
        self.prepare_admission(seat, lifecycle, is_replacement)
    }

    fn prepare_admission(
        &self,
        seat: SeatId,
        lifecycle: RosterMatchLifecycle,
        is_replacement: bool,
    ) -> Result<PreparedRosterAdmission, RosterRuntimeError> {
        let revision = self.next_revision()?;
        let room =
            publication::room_view(&self.identity, &lifecycle, revision, self.settings_revision)?;
        let body = self.sim.as_ref().filter(|_| {
            matches!(
                lifecycle.phase(),
                MatchPhase::Running | MatchPhase::Result { .. } | MatchPhase::MatchResult { .. }
            )
        });
        let capture = body
            .map(|sim| {
                v14::capture_expanded(&sim.state, &[]).map_err(|_| RosterRuntimeError::Publication)
            })
            .transpose()?;
        let state_epoch = body
            .map(|sim| publication::counter64(sim.epoch))
            .transpose()?;
        let generation = body.map(|sim| sim.generation);
        let last_action_id = if state_epoch == Some(room.round_id) {
            self.inputs
                .last_manual_action(seat)
                .map(publication::counter)
                .transpose()?
        } else {
            None
        };
        let notice = if let Some(sim) = body {
            publication::snapshot(room.clone(), sim.epoch, &sim.state, &[], sim.generation)?
        } else {
            publication::view_notice(room.clone())
        };
        Ok(PreparedRosterAdmission {
            lifecycle,
            revision,
            seat,
            is_replacement,
            room,
            state_epoch,
            capture,
            generation,
            last_action_id,
            notice,
        })
    }

    pub(in crate::game::rooms) fn commit_admission(&mut self, prepared: PreparedRosterAdmission) {
        if prepared.is_replacement {
            self.stop_seat_inputs(prepared.seat);
        }
        self.cpu_publication = prepared.notice.cpu_publication.clone();
        self.lifecycle = prepared.lifecycle;
        self.revision = prepared.revision;
    }
}
