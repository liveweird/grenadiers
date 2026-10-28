use std::sync::Arc;

use grenadiers_core::{
    game::SeatId,
    protocol::{
        Counter,
        v13::{ExpandedCapture, RoomView, capture_expanded},
        v14,
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
    cpu_ready: bool,
    clear_round: bool,
    live14: bool,
    pub(in crate::game::rooms) room: RoomView,
    pub(in crate::game::rooms) state_epoch: Option<Counter>,
    pub(in crate::game::rooms) capture: Option<ExpandedCapture>,
    live_capture: Option<v14::ExpandedCapture>,
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
        self.prepare_admission(
            seat,
            lifecycle,
            AdmissionKind::Join,
            false,
            self.next_revision()?,
        )
    }

    pub(in crate::game::rooms) fn prepare_solo_creator(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRuntimeAdmission, MatchRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        lifecycle.occupy(seat).map_err(super::map_lifecycle_error)?;
        if !self.is_live14() {
            lifecycle
                .set_ready(SeatId::Two, true)
                .map_err(super::map_lifecycle_error)?;
        }
        let revision = self
            .revision
            .checked_add(if self.is_live14() { 1 } else { 2 })
            .and_then(|value| Counter::new(value).ok())
            .map(Counter::get)
            .ok_or(MatchRuntimeError::CapacityReached)?;
        self.prepare_admission(
            seat,
            lifecycle,
            AdmissionKind::Join,
            !self.is_live14(),
            revision,
        )
    }

    pub(in crate::game::rooms) fn prepare_resume(
        &self,
        seat: SeatId,
    ) -> Result<PreparedRuntimeAdmission, MatchRuntimeError> {
        let mut lifecycle = self.lifecycle.clone();
        let kind = match lifecycle
            .reconnect_grace()
            .seat(seat)
            .ok_or(MatchRuntimeError::InvalidPhase)?
        {
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
        self.prepare_admission(
            seat,
            lifecycle,
            AdmissionKind::Resume(kind),
            false,
            self.next_revision()?,
        )
    }

    fn prepare_admission(
        &self,
        seat: SeatId,
        mut lifecycle: MatchLifecycle,
        kind: AdmissionKind,
        cpu_ready: bool,
        revision: u32,
    ) -> Result<PreparedRuntimeAdmission, MatchRuntimeError> {
        let clear_round = self.settle_candidate_cpu(&mut lifecycle)?;
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
        let live_capture = if self.is_live14() {
            body.map(|sim| {
                v14::capture_expanded(&sim.state, &[]).map_err(|_| MatchRuntimeError::Publication)
            })
            .transpose()?
        } else {
            None
        };
        let generation = body.and_then(|sim| sim.generation_record);
        let state_epoch = body.map(|sim| wire_counter(sim.epoch)).transpose()?;
        let last_bomb_action_id = if state_epoch == Some(room.round_id) {
            self.inputs
                .last_manual_action(seat)
                .map(|value| wire_counter(u64::from(value)))
                .transpose()?
        } else {
            None
        };
        let notice = if let Some(sim) = body {
            if self.is_live14() {
                let frames = crate::game::rooms::legacy_v14::snapshot(
                    &room,
                    wire_counter(sim.epoch)?,
                    &sim.state,
                    &[],
                    sim.generation_record,
                )
                .map_err(|_| MatchRuntimeError::Publication)?;
                MatchNotice {
                    room: room.clone(),
                    frames: Some(super::super::SeatFrames::new(frames)),
                    cpu_observation: publication::cpu_observation(&self.identity, &room, sim, &[])?,
                }
            } else {
                publication::snapshot(
                    &self.identity,
                    &lifecycle,
                    revision,
                    self.settings_revision,
                    self.settings_editor,
                    sim,
                    &[],
                )?
            }
        } else {
            self.view_notice(room.clone())?
        };
        Ok(PreparedRuntimeAdmission {
            lifecycle,
            revision,
            seat,
            kind,
            cpu_ready,
            clear_round,
            live14: self.is_live14(),
            room,
            state_epoch,
            capture,
            live_capture,
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
        if prepared.clear_round {
            self.clear_round();
        }
        self.revision = prepared.revision;
        let tick = self.lifecycle.tick();
        match prepared.kind {
            AdmissionKind::Join => {
                self.capture(
                    tick,
                    crate::game::rooms::replay::Operation::Occupy(prepared.seat),
                );
                if prepared.cpu_ready {
                    self.capture_action(
                        tick,
                        SeatId::Two,
                        self.lifecycle.round_attempt(),
                        super::super::MatchAction::ReadyChecked {
                            ready: true,
                            settings_revision: self.settings_revision,
                        },
                    );
                }
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

impl PreparedRuntimeAdmission {
    pub(in crate::game::rooms) fn encode(
        &self,
        seat: SeatId,
        sequence: Counter,
        token: grenadiers_core::protocol::v13::ResumeToken,
    ) -> Result<Arc<str>, MatchRuntimeError> {
        let encoded = if self.live14 {
            crate::game::rooms::legacy_v14::admission(
                &self.room,
                self.state_epoch,
                self.live_capture.as_ref(),
                self.generation,
                seat,
                self.last_bomb_action_id,
                sequence,
                token,
            )
            .map_err(|_| MatchRuntimeError::Publication)?
        } else {
            grenadiers_core::protocol::v13::prepare_admission(
                grenadiers_core::protocol::v13::AdmissionProjection {
                    room: &self.room,
                    state_epoch: self.state_epoch,
                    capture: self.capture.as_ref(),
                    generation: self.generation,
                },
                grenadiers_core::protocol::v13::PersonalAdmission {
                    recipient: super::super::publication::wire_seat(seat)?,
                    last_action_id: self.last_bomb_action_id,
                    sequence,
                    resume_token: token,
                },
            )
            .map_err(|_| MatchRuntimeError::Publication)?
        };
        Ok(Arc::from(encoded))
    }
}

fn wire_counter(value: u64) -> Result<Counter, MatchRuntimeError> {
    u32::try_from(value)
        .ok()
        .and_then(|value| Counter::new(value).ok())
        .ok_or(MatchRuntimeError::CapacityReached)
}
