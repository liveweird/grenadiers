use grenadiers_core::{
    game::SeatId,
    protocol::RoomId,
    room::{DepartureReason, InterruptReason, MatchPhase},
};
use tokio::sync::{oneshot, watch};
use tokio::time::Instant;

use super::super::{
    match_runtime::{MatchAction, MatchNotice, MatchRuntime, MatchRuntimeError},
    types::{BindingKey, DepartureGuard, RoomError},
};
use super::types::{
    ClosingReason, MatchAdmission, MatchOwnerNotice, OwnerCommand, PendingDeparture, SeatSlot,
    map_runtime_error, wire_seat,
};

pub(super) struct Driver {
    pub(super) runtime: MatchRuntime,
    room_id: RoomId,
    notices: watch::Sender<MatchOwnerNotice>,
    pub(super) last: MatchNotice,
    pub(super) seats: [Option<SeatSlot>; 2],
    pub(super) terminal_at: Option<Instant>,
    pub(super) lobby_deadline: Option<Instant>,
}

impl Driver {
    pub(super) fn new(
        runtime: MatchRuntime,
        initial: MatchNotice,
        notices: watch::Sender<MatchOwnerNotice>,
        lobby_deadline: Instant,
    ) -> Self {
        Self {
            runtime,
            room_id: initial.room.room_id.clone(),
            notices,
            last: initial,
            seats: [None, None],
            terminal_at: None,
            lobby_deadline: Some(lobby_deadline),
        }
    }

    pub(super) fn command(
        &mut self,
        command: OwnerCommand,
        now: Instant,
    ) -> Result<(), MatchRuntimeError> {
        match command {
            OwnerCommand::Join { binding, reply } => self.join(binding, reply),
            OwnerCommand::Action {
                binding,
                room_id,
                round_id,
                action,
                reply,
            } => {
                let result = self.action(binding, &room_id, round_id, action, now);
                match result {
                    Ok(()) => {
                        let _ = reply.send(Ok(()));
                        Ok(())
                    }
                    Err(error) if fatal(error) => {
                        let _ = reply.send(Err(map_runtime_error(error)));
                        Err(error)
                    }
                    Err(error) => {
                        let _ = reply.send(Err(map_runtime_error(error)));
                        Ok(())
                    }
                }
            }
            OwnerCommand::Leave {
                binding,
                room_id,
                round_id,
                reply,
            } => {
                self.leave(binding, &room_id, round_id, reply);
                Ok(())
            }
        }
    }

    fn join(
        &mut self,
        binding: BindingKey,
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
                .any(|slot| slot.binding == binding)
        {
            let _ = reply.send(Err(RoomError::RoomFull));
            return Ok(());
        }
        let Some(index) = self.seats.iter().position(Option::is_none) else {
            let _ = reply.send(Err(RoomError::RoomFull));
            return Ok(());
        };
        let seat = SeatId::ALL[index];
        let notice = match self.runtime.occupy(seat) {
            Ok(notice) => notice,
            Err(error) => {
                let _ = reply.send(Err(map_runtime_error(error)));
                return if fatal(error) { Err(error) } else { Ok(()) };
            }
        };
        let (departure_sender, departure) = oneshot::channel();
        self.seats[index] = Some(SeatSlot {
            binding,
            departure,
            pending: None,
        });
        self.publish(notice.clone());
        let admission = MatchAdmission {
            room_id: self.room_id.clone(),
            round_id: notice.room.round_id.get() as u64,
            seat: wire_seat(seat),
            view: notice.room,
            notices: self.notices.subscribe(),
            guard: DepartureGuard::new(departure_sender),
        };
        if let Err(admission) = reply.send(Ok(admission)) {
            drop(admission);
        }
        Ok(())
    }

    fn action(
        &mut self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
        action: MatchAction,
        now: Instant,
    ) -> Result<(), MatchRuntimeError> {
        let seat = self
            .validate(binding, room_id, round_id)
            .map_err(|_| MatchRuntimeError::StaleEpoch)?;
        if matches!(action, MatchAction::Rematch(_)) && self.has_pending() {
            return Err(MatchRuntimeError::StaleEpoch);
        }
        let was_terminal = matches!(
            self.runtime.lifecycle().phase(),
            MatchPhase::MatchResult { .. }
        );
        if let Some(notice) = self.runtime.action(seat, round_id, action)? {
            self.publish(notice);
        }
        if was_terminal && self.runtime.lifecycle().phase() == MatchPhase::Waiting {
            self.terminal_at = None;
            self.lobby_deadline = Some(now + super::runner::LOBBY_LIFETIME);
        }
        self.observe_phase(now);
        Ok(())
    }

    fn leave(
        &mut self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
        reply: oneshot::Sender<Result<(), RoomError>>,
    ) {
        if matches!(
            self.runtime.lifecycle().phase(),
            MatchPhase::Interrupted { .. }
        ) {
            let _ = reply.send(Err(RoomError::InvalidPhase));
            return;
        }
        let Ok(seat) = self.validate(binding, room_id, round_id) else {
            let _ = reply.send(Err(RoomError::StaleRoom));
            return;
        };
        self.runtime.stop_seat_inputs(seat);
        self.seats[seat.index()]
            .as_mut()
            .expect("validated slot")
            .pending = Some(PendingDeparture {
            binding,
            reason: DepartureReason::Explicit,
            reply: Some(reply),
        });
    }

    fn validate(
        &self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
    ) -> Result<SeatId, RoomError> {
        if room_id != &self.room_id || round_id != self.runtime.lifecycle().round_attempt() {
            return Err(RoomError::StaleRoom);
        }
        self.seats
            .iter()
            .enumerate()
            .find_map(|(index, slot)| {
                slot.as_ref()
                    .filter(|slot| slot.binding == binding && slot.pending.is_none())
                    .map(|_| SeatId::ALL[index])
            })
            .ok_or(RoomError::StaleRoom)
    }

    pub(super) fn inspect_departures(&mut self) {
        for (index, slot) in self.seats.iter_mut().enumerate() {
            let Some(slot) = slot.as_mut() else { continue };
            if slot.pending.is_some() {
                continue;
            }
            match slot.departure.try_recv() {
                Ok(reason) => {
                    self.runtime.stop_seat_inputs(SeatId::ALL[index]);
                    slot.pending = Some(PendingDeparture {
                        binding: slot.binding,
                        reason,
                        reply: None,
                    });
                }
                Err(oneshot::error::TryRecvError::Empty | oneshot::error::TryRecvError::Closed) => {
                }
            }
        }
    }

    pub(super) fn commit_departures(&mut self, now: Instant) -> Result<bool, MatchRuntimeError> {
        if !self.has_pending() {
            return Ok(false);
        }
        let departures = self.pending_reasons();
        if let Some(notice) = self.runtime.depart_batch(departures)? {
            self.publish(notice);
        }
        self.finish_pending();
        self.observe_phase(now);
        Ok(true)
    }

    pub(super) fn interrupt_and_commit(
        &mut self,
        reason: InterruptReason,
        now: Instant,
    ) -> Result<bool, MatchRuntimeError> {
        if !self.has_pending() {
            return self.interrupt(reason, now);
        }
        let notice = self
            .runtime
            .interrupt_and_depart(reason, self.pending_reasons())?;
        self.publish(notice);
        self.finish_pending();
        self.observe_phase(now);
        Ok(true)
    }

    fn pending_reasons(&self) -> [Option<DepartureReason>; 2] {
        self.seats.each_ref().map(|slot| {
            slot.as_ref()
                .and_then(|slot| slot.pending.as_ref().map(|pending| pending.reason))
        })
    }

    fn finish_pending(&mut self) {
        for slot in &mut self.seats {
            let remove = slot.as_ref().is_some_and(|slot| slot.pending.is_some());
            if remove {
                let mut removed = slot.take().expect("pending slot");
                let pending = removed.pending.take().expect("pending departure");
                debug_assert_eq!(pending.binding, removed.binding);
                if let Some(reply) = pending.reply {
                    let _ = reply.send(Ok(()));
                }
            }
        }
    }

    pub(super) fn tick(&mut self, now: Instant) -> Result<(), MatchRuntimeError> {
        if let Some(notice) = self.runtime.tick()? {
            self.publish(notice);
        }
        self.observe_phase(now);
        Ok(())
    }

    pub(super) fn interrupt(
        &mut self,
        reason: InterruptReason,
        now: Instant,
    ) -> Result<bool, MatchRuntimeError> {
        let published = if let Some(notice) = self.runtime.interrupt(reason)? {
            self.publish(notice);
            true
        } else {
            false
        };
        self.observe_phase(now);
        Ok(published)
    }

    pub(super) fn closing(&mut self, reason: ClosingReason) {
        let last = self
            .runtime
            .current_notice()
            .unwrap_or_else(|_| self.last.clone());
        self.last = last.clone();
        self.notices
            .send_replace(MatchOwnerNotice::Closing { last, reason });
        self.reject_waiters(RoomError::RoomNotFound);
    }

    pub(super) fn fail(&mut self) {
        self.close_retained(ClosingReason::InternalFailure);
        self.reject_waiters(RoomError::RoomNotFound);
    }

    pub(super) fn settle_shutdown(&mut self, now: Instant) {
        let published = self
            .interrupt_and_commit(InterruptReason::ServerStopped, now)
            .unwrap_or(false);
        if published {
            self.close_retained(ClosingReason::ServerStopped);
            self.reject_waiters(RoomError::RoomNotFound);
        } else {
            self.closing(ClosingReason::ServerStopped);
        }
    }

    pub(super) fn has_pending(&self) -> bool {
        self.seats
            .iter()
            .flatten()
            .any(|slot| slot.pending.is_some())
    }

    pub(super) fn is_empty(&self) -> bool {
        self.seats.iter().all(Option::is_none)
    }

    fn publish(&mut self, notice: MatchNotice) {
        self.last = notice.clone();
        self.notices
            .send_replace(MatchOwnerNotice::Publication(notice));
    }

    fn close_retained(&mut self, reason: ClosingReason) {
        self.notices.send_replace(MatchOwnerNotice::Closing {
            last: self.last.clone(),
            reason,
        });
    }

    fn observe_phase(&mut self, now: Instant) {
        if self.runtime.lifecycle().series().actual_round_number() > 0 {
            self.lobby_deadline = None;
        }
        if matches!(
            self.runtime.lifecycle().phase(),
            MatchPhase::MatchResult { .. } | MatchPhase::Interrupted { .. }
        ) {
            self.terminal_at.get_or_insert(now);
        }
    }

    fn reject_waiters(&mut self, error: RoomError) {
        for slot in self.seats.iter_mut().flatten() {
            if let Some(mut pending) = slot.pending.take()
                && let Some(reply) = pending.reply.take()
            {
                let _ = reply.send(Err(error));
            }
        }
    }
}

const fn fatal(error: MatchRuntimeError) -> bool {
    matches!(
        error,
        MatchRuntimeError::CapacityReached
            | MatchRuntimeError::InvalidInterrupt
            | MatchRuntimeError::Publication
    )
}
