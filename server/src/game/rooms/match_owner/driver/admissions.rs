use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, RoomId, v6::ResumeToken},
};
use tokio::sync::oneshot;

use super::{Driver, fatal};
use crate::game::rooms::{
    match_owner::types::{LiveBinding, MatchAdmission, SeatSlot, map_runtime_error, wire_seat},
    match_runtime::MatchRuntimeError,
    resume::{BindingFence, ResumeCredential},
    types::{BindingKey, DepartureGuard, RoomError},
};

impl Driver {
    pub(super) fn join(
        &mut self,
        binding: BindingKey,
        sequence: Counter,
        reply: oneshot::Sender<Result<MatchAdmission, RoomError>>,
    ) -> Result<(), MatchRuntimeError> {
        if reply.is_closed() {
            return Ok(());
        }
        if self.has_pending()
            || self
                .seats
                .iter()
                .flatten()
                .filter_map(|slot| slot.live.as_ref())
                .any(|live| live.fence.matches(binding))
        {
            let _ = reply.send(Err(RoomError::RoomFull));
            return Ok(());
        }
        let Some(index) = self.seats.iter().position(Option::is_none) else {
            let _ = reply.send(Err(RoomError::RoomFull));
            return Ok(());
        };
        let seat = SeatId::ALL[index];
        let other = self
            .seats
            .iter()
            .flatten()
            .find_map(|slot| slot.credential.as_ref());
        let credential = match ResumeCredential::mint(self.random_fill.as_ref(), other) {
            Ok(credential) => credential,
            Err(_) => {
                let _ = reply.send(Err(RoomError::CapacityReached));
                return Ok(());
            }
        };
        let prepared = match self.runtime.prepare_join(seat) {
            Ok(prepared) => prepared,
            Err(error) => {
                let _ = reply.send(Err(map_runtime_error(error)));
                return if fatal(error) { Err(error) } else { Ok(()) };
            }
        };
        let (departure_sender, departure) = oneshot::channel();
        let (replaced, replaced_receiver) = oneshot::channel();
        let admission = MatchAdmission {
            room_id: self.room_id.clone(),
            round_id: u64::from(prepared.room.round_id.get()),
            seat: wire_seat(seat),
            sequence,
            room: prepared.room.clone(),
            resume_token: credential.token(),
            last_bomb_action_id: prepared.last_bomb_action_id,
            state: prepared.state.clone(),
            notices: self.notices.subscribe(),
            guard: DepartureGuard::new(departure_sender),
            replaced: replaced_receiver,
        };
        if reply.send(Ok(admission)).is_err() {
            return Ok(());
        }
        let notice = prepared.notice.clone();
        self.runtime.commit_admission(prepared);
        self.seats[index] = Some(SeatSlot {
            credential: Some(credential),
            live: Some(LiveBinding {
                fence: BindingFence::new(binding),
                departure,
                replaced: Some(replaced),
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
        reply: oneshot::Sender<Result<MatchAdmission, RoomError>>,
    ) -> Result<(), MatchRuntimeError> {
        if reply.is_closed() || room_id != &self.room_id || self.has_pending() {
            let _ = reply.send(Err(RoomError::ResumeRejected));
            return Ok(());
        }
        let Some(index) = self.seats.iter().position(|slot| {
            slot.as_ref()
                .and_then(|slot| slot.credential.as_ref())
                .is_some_and(|credential| credential.matches(token))
        }) else {
            let _ = reply.send(Err(RoomError::ResumeRejected));
            return Ok(());
        };
        let seat = SeatId::ALL[index];
        let prepared = match self.runtime.prepare_resume(seat) {
            Ok(prepared) => prepared,
            Err(_) => {
                let _ = reply.send(Err(RoomError::ResumeRejected));
                return Ok(());
            }
        };
        let (departure_sender, departure) = oneshot::channel();
        let (replaced, replaced_receiver) = oneshot::channel();
        let credential = self.seats[index]
            .as_ref()
            .and_then(|slot| slot.credential.as_ref())
            .expect("matched credential");
        let admission = MatchAdmission {
            room_id: self.room_id.clone(),
            round_id: u64::from(prepared.room.round_id.get()),
            seat: wire_seat(seat),
            sequence,
            room: prepared.room.clone(),
            resume_token: credential.token(),
            last_bomb_action_id: prepared.last_bomb_action_id,
            state: prepared.state.clone(),
            notices: self.notices.subscribe(),
            guard: DepartureGuard::new(departure_sender),
            replaced: replaced_receiver,
        };
        if reply.send(Ok(admission)).is_err() {
            return Ok(());
        }
        let notice = prepared.notice.clone();
        self.runtime.commit_admission(prepared);
        let slot = self.seats[index].as_mut().expect("matched slot");
        let fence = if let Some(mut old) = slot.live.take() {
            let expected = old.fence.current();
            debug_assert!(old.fence.replace_if_current(expected, binding));
            if let Some(replaced) = old.replaced.take() {
                let _ = replaced.send(());
            }
            old.fence
        } else {
            BindingFence::new(binding)
        };
        slot.live = Some(LiveBinding {
            fence,
            departure,
            replaced: Some(replaced),
            pending: None,
        });
        self.publish(notice);
        Ok(())
    }
}
