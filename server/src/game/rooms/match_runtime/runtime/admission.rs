use grenadiers_core::{
    game::SeatId,
    protocol::{
        Counter,
        v9::{ExpandedCapture, RoomView, capture_expanded},
    },
    room::{MatchLifecycle, reconnect::ReservationState},
};

use super::{MatchNotice, MatchRuntime};
use crate::game::rooms::match_runtime::{MatchRuntimeError, publication};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ResumeKind {
    Restore,
    Replace,
}

pub(in crate::game::rooms) struct PreparedRuntimeAdmission {
    lifecycle: MatchLifecycle,
    revision: u32,
    seat: SeatId,
    kind: AdmissionKind,
    pub(in crate::game::rooms) room: RoomView,
    pub(in crate::game::rooms) state_epoch: Option<Counter>,
    pub(in crate::game::rooms) capture: Option<ExpandedCapture>,
    pub(in crate::game::rooms) generation: Option<grenadiers_core::game::GenerationRecord>,
    pub(in crate::game::rooms) last_bomb_action_id: Option<Counter>,
    pub(in crate::game::rooms) notice: MatchNotice,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdmissionKind {
    Join,
    Resume(ResumeKind),
}

impl MatchRuntime {
    pub(in crate::game::rooms) fn prepare_join(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRuntimeAdmission, MatchRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        lifecycle.occupy(seat).map_err(super::map_lifecycle_error)?;
        self.prepare_admission(seat, lifecycle, AdmissionKind::Join)
    }

    pub(in crate::game::rooms) fn prepare_resume(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRuntimeAdmission, MatchRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        let kind = match lifecycle.reconnect_grace().seat(seat) {
            ReservationState::Disconnected { .. } => {
                lifecycle
                    .resume_connection(seat)
                    .map_err(super::map_lifecycle_error)?;
                ResumeKind::Restore
            }
            ReservationState::Connected => ResumeKind::Replace,
            ReservationState::Vacant | ReservationState::Expired => {
                return Err(MatchRuntimeError::StaleEpoch);
            }
        };
        self.prepare_admission(seat, lifecycle, AdmissionKind::Resume(kind))
    }

    fn prepare_admission(
        &self,
        seat: SeatId,
        lifecycle: MatchLifecycle,
        kind: AdmissionKind,
    ) -> Result<PreparedRuntimeAdmission, MatchRuntimeError> {
        let revision = self.next_revision()?;
        let room = publication::room_view(
            &self.identity,
            &lifecycle,
            revision,
            self.settings_revision,
            self.settings_editor,
        )?;
        if self.sim.as_ref().is_some_and(|sim| {
            sim.state.rules() != &self.identity.profile.rules_config(lifecycle.settings())
        }) {
            return Err(MatchRuntimeError::Publication);
        }
        let body = self
            .sim
            .as_ref()
            .filter(|_| publication::phase_has_body(&room.phase));
        let capture = body
            .map(|sim| {
                capture_expanded(&sim.state, &[]).map_err(|_| MatchRuntimeError::Publication)
            })
            .transpose()?;
        let generation = body.and_then(|sim| sim.generation_record);
        let state_epoch = body.map(|sim| wire_counter(sim.epoch)).transpose()?;
        let last_bomb_action_id = if state_epoch == Some(room.round_id) {
            self.inputs
                .last_bomb_action(seat)
                .map(|value| wire_counter(u64::from(value)))
                .transpose()?
        } else {
            None
        };
        let notice = if let Some(sim) = body {
            publication::snapshot(
                &self.identity,
                &lifecycle,
                revision,
                self.settings_revision,
                self.settings_editor,
                sim,
                &[],
            )?
        } else {
            MatchNotice {
                room: room.clone(),
                frames: None,
            }
        };
        Ok(PreparedRuntimeAdmission {
            lifecycle,
            revision,
            seat,
            kind,
            room,
            state_epoch,
            capture,
            generation,
            last_bomb_action_id,
            notice,
        })
    }

    pub(in crate::game::rooms) fn commit_admission(
        &mut self,
        prepared: PreparedRuntimeAdmission,
    ) -> ResumeKind {
        if prepared.kind == AdmissionKind::Resume(ResumeKind::Replace) {
            self.stop_seat_inputs(prepared.seat);
        }
        self.lifecycle = prepared.lifecycle;
        self.revision = prepared.revision;
        let tick = self.lifecycle.tick();
        match prepared.kind {
            AdmissionKind::Join => {
                self.capture(
                    tick,
                    crate::game::rooms::replay::Operation::Occupy(prepared.seat),
                );
                ResumeKind::Restore
            }
            AdmissionKind::Resume(kind) => {
                self.capture(
                    tick,
                    crate::game::rooms::replay::Operation::Resume {
                        seat: prepared.seat,
                        kind,
                    },
                );
                kind
            }
        }
    }
}

fn wire_counter(value: u64) -> Result<Counter, MatchRuntimeError> {
    u32::try_from(value)
        .ok()
        .and_then(|value| Counter::new(value).ok())
        .ok_or(MatchRuntimeError::CapacityReached)
}
