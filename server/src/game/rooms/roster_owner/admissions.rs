use std::sync::Arc;

use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, RoomId, v14::ResumeToken},
    room::RosterParticipant,
};
use tokio::sync::oneshot;

use super::{
    driver::{Driver, fatal},
    types::{LiveBinding, RosterAdmission, SeatSlot, map_runtime_error},
};
use crate::game::rooms::{
    resume::{BindingFence, ResumeCredential},
    roster_runtime::{PreparedRosterAdmission, RosterRuntimeError},
    types::{BindingKey, DepartureGuard, RoomError},
};

impl Driver {
    pub(super) fn join(
        &mut self,
        creator: bool,
        binding: BindingKey,
        sequence: Counter,
        reply: oneshot::Sender<Result<RosterAdmission, RoomError>>,
    ) -> Result<(), RosterRuntimeError> {
        if reply.is_closed() {
            return Ok(());
        }
        if self.has_pending() || self.binding_exists(binding) {
            let _ = reply.send(Err(RoomError::RoomFull));
            return Ok(());
        }
        let seat = if creator {
            self.joinable(SeatId::One).then_some(SeatId::One)
        } else {
            self.runtime
                .identity()
                .configuration
                .human_seats()
                .find(|&seat| self.joinable(seat))
        };
        let Some(seat) = seat else {
            let _ = reply.send(Err(RoomError::RoomFull));
            return Ok(());
        };
        let retained: Vec<_> = self
            .seats
            .iter()
            .flatten()
            .map(|slot| &slot.credential)
            .collect();
        let credential = match ResumeCredential::mint_unique(self.random_fill.as_ref(), &retained) {
            Ok(value) => value,
            Err(_) => {
                let _ = reply.send(Err(RoomError::CapacityReached));
                return Ok(());
            }
        };
        let prepared = match self.runtime.prepare_join(seat) {
            Ok(value) => value,
            Err(error) => return self.reject_runtime(reply, error),
        };
        let binding_parts = make_binding();
        let frame = match prepared.encode(sequence, credential.token()) {
            Ok(frame) => Arc::from(frame),
            Err(error) => return self.reject_runtime(reply, error),
        };
        let admission = admission(
            &prepared,
            seat,
            credential.token(),
            frame,
            self.notices.subscribe(),
            binding_parts.0,
            binding_parts.2,
        );
        if reply.send(Ok(admission)).is_err() {
            return Ok(());
        }
        let notice = prepared.notice.clone();
        self.runtime.commit_admission(prepared);
        self.seats[seat.index()] = Some(SeatSlot {
            credential,
            live: Some(LiveBinding {
                fence: BindingFence::new(binding),
                departure: binding_parts.1,
                replaced: Some(binding_parts.3),
                pending: None,
            }),
        });
        self.publish(notice);
        Ok(())
    }

    pub(super) fn resume(
        &mut self,
        binding: BindingKey,
        sequence: Counter,
        room_id: &RoomId,
        token: &ResumeToken,
        reply: oneshot::Sender<Result<RosterAdmission, RoomError>>,
    ) -> Result<(), RosterRuntimeError> {
        if reply.is_closed() || room_id != &self.room_id || self.has_pending() {
            let _ = reply.send(Err(RoomError::ResumeRejected));
            return Ok(());
        }
        let Some(index) = self.seats.iter().position(|slot| {
            slot.as_ref()
                .is_some_and(|slot| slot.credential.matches(token))
        }) else {
            let _ = reply.send(Err(RoomError::ResumeRejected));
            return Ok(());
        };
        let seat = SeatId::ALL[index];
        let prepared = match self.runtime.prepare_resume(seat) {
            Ok(value) => value,
            Err(_) => {
                let _ = reply.send(Err(RoomError::ResumeRejected));
                return Ok(());
            }
        };
        let parts = make_binding();
        let credential = &self.seats[index].as_ref().unwrap().credential;
        let frame = match prepared.encode(sequence, credential.token()) {
            Ok(frame) => Arc::from(frame),
            Err(_) => {
                let _ = reply.send(Err(RoomError::ResumeRejected));
                return Ok(());
            }
        };
        let admission = admission(
            &prepared,
            seat,
            credential.token(),
            frame,
            self.notices.subscribe(),
            parts.0,
            parts.2,
        );
        if reply.send(Ok(admission)).is_err() {
            return Ok(());
        }
        let notice = prepared.notice.clone();
        self.runtime.commit_admission(prepared);
        let slot = self.seats[index].as_mut().unwrap();
        let fence = if let Some(mut old) = slot.live.take() {
            let expected = old.fence.current();
            let _ = old.fence.replace_if_current(expected, binding);
            if let Some(replaced) = old.replaced.take() {
                let _ = replaced.send(());
            }
            old.fence
        } else {
            BindingFence::new(binding)
        };
        slot.live = Some(LiveBinding {
            fence,
            departure: parts.1,
            replaced: Some(parts.3),
            pending: None,
        });
        self.publish(notice);
        Ok(())
    }

    fn joinable(&self, seat: SeatId) -> bool {
        self.seats[seat.index()].is_none()
            && self.runtime.lifecycle().participant(seat).ok()
                == Some(RosterParticipant::NeverAdmitted)
    }

    fn binding_exists(&self, binding: BindingKey) -> bool {
        self.seats.iter().flatten().any(|slot| {
            slot.live
                .as_ref()
                .is_some_and(|live| live.fence.matches(binding))
        })
    }

    fn reject_runtime<T>(
        &self,
        reply: oneshot::Sender<Result<T, RoomError>>,
        error: RosterRuntimeError,
    ) -> Result<(), RosterRuntimeError> {
        let _ = reply.send(Err(map_runtime_error(error)));
        if fatal(error) { Err(error) } else { Ok(()) }
    }
}

fn admission(
    prepared: &PreparedRosterAdmission,
    seat: SeatId,
    _resume_token: ResumeToken,
    personal_frame: Arc<str>,
    notices: tokio::sync::watch::Receiver<super::types::RosterOwnerNotice>,
    guard: DepartureGuard,
    replaced: oneshot::Receiver<()>,
) -> RosterAdmission {
    let (room_id, _round_id) = prepared.room_metadata();
    #[cfg(test)]
    let (room_code, revision) = prepared.test_metadata();
    RosterAdmission {
        room_id: room_id.clone(),
        #[cfg(test)]
        round_id: _round_id,
        seat,
        #[cfg(test)]
        room_code: room_code.clone(),
        #[cfg(test)]
        revision,
        #[cfg(test)]
        resume_token: _resume_token,
        personal_frame,
        notices,
        guard,
        replaced,
    }
}

fn make_binding() -> (
    DepartureGuard,
    oneshot::Receiver<grenadiers_core::room::DepartureReason>,
    oneshot::Receiver<()>,
    oneshot::Sender<()>,
) {
    let (departure_sender, departure) = oneshot::channel();
    let (replaced, replaced_receiver) = oneshot::channel();
    (
        DepartureGuard::new(departure_sender),
        departure,
        replaced_receiver,
        replaced,
    )
}
