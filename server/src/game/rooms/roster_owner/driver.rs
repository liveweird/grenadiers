mod cpu;

use std::sync::Arc;

use grenadiers_core::{
    game::SeatId,
    protocol::RoomId,
    room::{DepartureReason, InterruptReason, MatchPhase, RosterParticipant},
};
use tokio::{
    sync::{oneshot, watch},
    time::Instant,
};

use super::super::{
    ClosingReason, RandomFill,
    cpu::CpuController,
    roster_runtime::{RosterAction, RosterNotice, RosterRuntime, RosterRuntimeError},
    types::{BindingKey, RoomError},
};
use super::types::{
    OwnerCommand, PendingDeparture, RosterOwnerNotice, SeatSlot, map_runtime_error,
};

pub(super) struct Driver {
    pub(super) runtime: RosterRuntime,
    pub(super) room_id: RoomId,
    pub(super) notices: watch::Sender<RosterOwnerNotice>,
    last: RosterNotice,
    pub(super) seats: [Option<SeatSlot>; 4],
    pub(super) retired_credentials: Vec<super::super::resume::ResumeCredential>,
    pub(super) terminal_at: Option<Instant>,
    pub(super) lobby_deadline: Option<Instant>,
    pub(super) random_fill: Arc<RandomFill>,
    pub(super) cpus: Vec<CpuController>,
    pub(super) cpu_stop_at: [Option<grenadiers_core::game::Tick>; 4],
    pub(super) cpu_stationary: [bool; 4],
    pub(super) next_cpu: usize,
}

impl Driver {
    pub(super) fn new(
        runtime: RosterRuntime,
        initial: RosterNotice,
        notices: watch::Sender<RosterOwnerNotice>,
        lobby_deadline: Instant,
        random_fill: Arc<RandomFill>,
        cpus: Vec<CpuController>,
    ) -> Self {
        Self {
            room_id: initial
                .legacy_room()
                .map(|room| &room.room_id)
                .or_else(|| initial.environment_room().map(|room| &room.room_id))
                .or_else(|| initial.lair_room().map(|room| &room.room_id))
                .or_else(|| initial.respawn_room().map(|room| &room.room_id))
                .expect("owner requires a checked room publication")
                .clone(),
            runtime,
            notices,
            last: initial,
            seats: [None, None, None, None],
            retired_credentials: Vec::new(),
            terminal_at: None,
            lobby_deadline: Some(lobby_deadline),
            random_fill,
            cpus,
            cpu_stop_at: [None; 4],
            cpu_stationary: [true; 4],
            next_cpu: 0,
        }
    }

    pub(super) fn command(
        &mut self,
        command: OwnerCommand,
        now: Instant,
    ) -> Result<(), RosterRuntimeError> {
        match command {
            OwnerCommand::Join {
                creator,
                binding,
                sequence,
                reply,
            } => self.join(creator, binding, sequence, reply),
            OwnerCommand::Resume {
                binding,
                sequence,
                room_id,
                token,
                reply,
            } => self.resume(binding, sequence, &room_id, &token, reply),
            OwnerCommand::Action {
                binding,
                room_id,
                round_id,
                action,
                reply,
            } => {
                let result = self.action(binding, &room_id, round_id, action, now);
                let fatal_error = result.err().filter(|error| fatal(*error));
                let _ = reply.send(result.map_err(map_runtime_error));
                fatal_error.map_or(Ok(()), Err)
            }
            OwnerCommand::LairSettings {
                binding,
                room_id,
                round_id,
                action,
                reply,
            } => {
                let result = self.lair_settings(binding, &room_id, round_id, action, now);
                let fatal_error = result.err().filter(|error| fatal(*error));
                let _ = reply.send(result.map_err(map_runtime_error));
                fatal_error.map_or(Ok(()), Err)
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

    fn action(
        &mut self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
        action: RosterAction,
        now: Instant,
    ) -> Result<(), RosterRuntimeError> {
        let seat = self
            .validate(binding, room_id, round_id)
            .map_err(|_| RosterRuntimeError::StaleEpoch)?;
        if matches!(action, RosterAction::Rematch(_)) && self.has_pending() {
            return Err(RosterRuntimeError::StaleEpoch);
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

    fn lair_settings(
        &mut self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
        action: super::super::live_registry::LairSettingsAction,
        now: Instant,
    ) -> Result<(), RosterRuntimeError> {
        let seat = self
            .validate(binding, room_id, round_id)
            .map_err(|_| RosterRuntimeError::StaleEpoch)?;
        if let Some(notice) = self.runtime.update_respawn_settings(
            seat,
            round_id,
            action.settings_revision,
            action.settings,
            action.cloud,
            action.tuning,
        )? {
            self.publish(notice);
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
        let Ok(seat) = self.validate(binding, room_id, round_id) else {
            let _ = reply.send(Err(RoomError::StaleRoom));
            return;
        };
        self.runtime.stop_seat_inputs(seat);
        self.seats[seat.index()]
            .as_mut()
            .unwrap()
            .live
            .as_mut()
            .unwrap()
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
                slot.as_ref()?
                    .live
                    .as_ref()
                    .filter(|live| live.pending.is_none() && live.fence.matches(binding))
                    .map(|_| SeatId::ALL[index])
            })
            .ok_or(RoomError::StaleRoom)
    }

    pub(super) fn inspect_departures(&mut self) {
        for (index, slot) in self.seats.iter_mut().enumerate() {
            let Some(live) = slot.as_mut().and_then(|slot| slot.live.as_mut()) else {
                continue;
            };
            if live.pending.is_some() {
                continue;
            }
            if let Ok(reason) = live.departure.try_recv() {
                self.runtime.stop_seat_inputs(SeatId::ALL[index]);
                live.pending = Some(PendingDeparture {
                    binding: live.fence.current(),
                    reason,
                    reply: None,
                });
            }
        }
    }

    fn pending_reasons(&self) -> [Option<DepartureReason>; 4] {
        self.seats.each_ref().map(|slot| {
            slot.as_ref()?
                .live
                .as_ref()?
                .pending
                .as_ref()
                .map(|pending| pending.reason)
        })
    }

    fn finish_pending(&mut self) {
        for slot in &mut self.seats {
            let Some(()) = slot
                .as_ref()
                .and_then(|slot| slot.live.as_ref())
                .and_then(|live| live.pending.as_ref())
                .map(|_| ())
            else {
                continue;
            };
            let mut live = slot.as_mut().unwrap().live.take().unwrap();
            let mut pending = live.pending.take().unwrap();
            debug_assert_eq!(pending.binding, live.fence.current());
            if let Some(reply) = pending.reply.take() {
                let _ = reply.send(Ok(()));
            }
        }
    }

    pub(super) fn tick(&mut self, now: Instant) -> Result<(), RosterRuntimeError> {
        self.service_cpu()?;
        if let Some(notice) = self.runtime.settle_boundary(self.pending_reasons())? {
            self.publish(notice);
        }
        self.finish_pending();
        self.retire_released_credentials();
        self.drop_expired_credentials();
        self.observe_phase(now);
        Ok(())
    }

    pub(super) fn interrupt_and_commit(
        &mut self,
        reason: InterruptReason,
        now: Instant,
    ) -> Result<bool, RosterRuntimeError> {
        let published = if self.has_pending() {
            let notice = self
                .runtime
                .interrupt_and_depart(reason, self.pending_reasons())?;
            self.publish(notice);
            self.finish_pending();
            true
        } else if let Some(notice) = self.runtime.interrupt(reason)? {
            self.publish(notice);
            true
        } else {
            false
        };
        self.observe_phase(now);
        Ok(published)
    }

    pub(super) fn publish(&mut self, notice: RosterNotice) {
        self.observe_cpus(&notice);
        self.last = notice.clone();
        self.notices
            .send_replace(RosterOwnerNotice::Publication(notice));
    }

    pub(super) fn is_empty(&self) -> bool {
        self.runtime.retained_human_count() == 0
    }
    pub(super) fn has_pending(&self) -> bool {
        self.seats.iter().flatten().any(|slot| {
            slot.live
                .as_ref()
                .is_some_and(|live| live.pending.is_some())
        })
    }

    pub(super) fn closing(&mut self, reason: ClosingReason) {
        let last = self
            .runtime
            .current_notice()
            .unwrap_or_else(|_| self.last.clone());
        self.last = last.clone();
        self.notices
            .send_replace(RosterOwnerNotice::Closing { last, reason });
        self.reject_waiters(RoomError::RoomNotFound);
    }

    pub(super) fn fail(&mut self) {
        self.closing(ClosingReason::InternalFailure);
    }
    pub(super) fn settle_shutdown(&mut self, now: Instant) {
        let _ = self.interrupt_and_commit(InterruptReason::ServerStopped, now);
        self.closing(ClosingReason::ServerStopped);
    }

    fn observe_phase(&mut self, now: Instant) {
        if self.runtime.lifecycle().actual_round_number() > 0 {
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
            if let Some(pending) = slot.live.as_mut().and_then(|live| live.pending.take())
                && let Some(reply) = pending.reply
            {
                let _ = reply.send(Err(error));
            }
        }
    }

    fn drop_expired_credentials(&mut self) {
        for (index, slot) in self.seats.iter_mut().enumerate() {
            if self
                .runtime
                .lifecycle()
                .participant(SeatId::ALL[index])
                .ok()
                == Some(RosterParticipant::Departed)
                && let Some(slot) = slot
            {
                slot.live = None;
            }
        }
    }
}

pub(super) const fn fatal(error: RosterRuntimeError) -> bool {
    matches!(
        error,
        RosterRuntimeError::CapacityReached
            | RosterRuntimeError::Publication
            | RosterRuntimeError::SeedUnavailable
    )
}
