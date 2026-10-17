use grenadiers_core::{
    game::{
        ActorInit, Direction, EntityId, Position, RulesConfig, SeatId, SpeedLevel, State, Tick,
        World, fixed_arena,
    },
    protocol::RoomId,
    room::{DepartureReason, InterruptReason, Phase, RoomLifecycle},
};
use tokio::sync::{oneshot, watch};
use tokio::time::Instant;

use super::owner::RoomCommand;
use super::{input::*, publication, types::*};

pub(super) struct Runtime {
    pub(super) identity: RoomIdentity,
    pub(super) lifecycle: RoomLifecycle,
    pub(super) revision: u32,
    pub(super) bindings: [Option<BindingKey>; 2],
    pub(super) departures: [Option<oneshot::Receiver<DepartureReason>>; 2],
    pub(super) notices: watch::Sender<RoomNotice>,
    pub(super) inputs: InputMailbox,
    pub(super) state: Option<State>,
    pub(super) reports: Vec<grenadiers_core::game::StepReport>,
    pub(super) terminal_at: Option<Instant>,
    pub(super) failed: bool,
}

impl Runtime {
    pub(super) fn new(identity: RoomIdentity, notices: watch::Sender<RoomNotice>) -> Self {
        Self {
            identity,
            lifecycle: RoomLifecycle::new(),
            revision: 0,
            bindings: [None; 2],
            departures: [None, None],
            notices,
            inputs: InputMailbox::default(),
            state: None,
            reports: Vec::new(),
            terminal_at: None,
            failed: false,
        }
    }

    pub(super) fn command(&mut self, command: RoomCommand) {
        match command {
            RoomCommand::Join { binding, reply } => self.join(binding, reply),
            RoomCommand::Leave {
                binding,
                room_id,
                round_id,
                reply,
            } => {
                if reply.is_closed() {
                    return;
                }
                let result = self.validate(binding, &room_id, round_id).map(|index| {
                    self.departures[index] = None;
                    self.bindings[index] = None;
                    let _ = self
                        .lifecycle
                        .depart(SeatId::ALL[index], DepartureReason::Explicit);
                    self.inputs.clear_pending();
                    self.after_lifecycle_change();
                });
                let _ = reply.send(result);
            }
            RoomCommand::Action {
                binding,
                room_id,
                round_id,
                action,
                reply,
            } => {
                if reply.is_closed() {
                    return;
                }
                let result = self.action(binding, &room_id, round_id, action);
                let _ = reply.send(result);
            }
        }
    }

    fn join(&mut self, binding: BindingKey, reply: oneshot::Sender<Result<Admission, RoomError>>) {
        if reply.is_closed() {
            return;
        }
        if self.lifecycle.phase() != Phase::Waiting {
            let _ = reply.send(Err(RoomError::RoomFull));
            return;
        }
        let Some(index) = self.bindings.iter().position(Option::is_none) else {
            let _ = reply.send(Err(RoomError::RoomFull));
            return;
        };
        let seat = SeatId::ALL[index];
        if self.lifecycle.occupy(seat).is_err() {
            let _ = reply.send(Err(RoomError::RoomFull));
            return;
        }
        self.bindings[index] = Some(binding);
        let (departure, receiver) = oneshot::channel();
        self.departures[index] = Some(receiver);
        let Some(view) = self.publish_view() else {
            self.rollback(index);
            let _ = reply.send(Err(RoomError::CapacityReached));
            return;
        };
        let admission = Admission {
            room_id: self.identity.id.clone(),
            round_id: self.lifecycle.round_attempt(),
            seat: publication::wire_seat(seat),
            view,
            notices: self.notices.subscribe(),
            guard: DepartureGuard::new(departure),
        };
        if let Err(Ok(admission)) = reply.send(Ok(admission)) {
            drop(admission);
            self.rollback(index);
            let _ = self.publish_view();
        }
    }

    fn action(
        &mut self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
        action: RoomAction,
    ) -> Result<(), RoomError> {
        let index = self.validate(binding, room_id, round_id)?;
        let seat = SeatId::ALL[index];
        match action {
            RoomAction::Ready(ready) => {
                if !matches!(
                    self.lifecycle.phase(),
                    Phase::Waiting | Phase::Countdown { .. }
                ) {
                    return Err(RoomError::InvalidPhase);
                }
                let old_attempt = self.lifecycle.round_attempt();
                let change = self
                    .lifecycle
                    .set_ready(seat, ready)
                    .map_err(|_| RoomError::InvalidPhase)?;
                if old_attempt != self.lifecycle.round_attempt() {
                    self.inputs.reset_round();
                }
                if change != grenadiers_core::room::Change::Unchanged {
                    self.publish_view().ok_or(RoomError::CapacityReached)?;
                }
            }
            RoomAction::Movement(direction) => {
                if self.lifecycle.phase() != Phase::Running {
                    return Err(RoomError::InvalidPhase);
                }
                self.inputs
                    .movement(seat, direction, self.lifecycle.tick())
                    .map_err(input_error)?;
            }
            RoomAction::Bomb(action_id) => {
                if self.lifecycle.phase() != Phase::Running {
                    return Err(RoomError::InvalidPhase);
                }
                self.inputs.bomb(seat, action_id).map_err(input_error)?;
            }
        }
        Ok(())
    }

    fn validate(
        &self,
        binding: BindingKey,
        room_id: &RoomId,
        round_id: u64,
    ) -> Result<usize, RoomError> {
        if room_id != &self.identity.id || round_id != self.lifecycle.round_attempt() {
            return Err(RoomError::StaleRoom);
        }
        self.bindings
            .iter()
            .position(|candidate| *candidate == Some(binding))
            .ok_or(RoomError::StaleRoom)
    }

    pub(super) fn depart(&mut self, index: usize, reason: DepartureReason) {
        self.departures[index] = None;
        self.bindings[index] = None;
        let _ = self.lifecycle.depart(SeatId::ALL[index], reason);
        self.inputs.clear_pending();
        self.after_lifecycle_change();
    }

    pub(super) fn drain_departures(&mut self) {
        for index in 0..self.departures.len() {
            let reason = match self.departures[index]
                .as_mut()
                .map(oneshot::Receiver::try_recv)
            {
                Some(Ok(reason)) => Some(reason),
                Some(Err(oneshot::error::TryRecvError::Closed)) => {
                    Some(DepartureReason::TransportLoss)
                }
                Some(Err(oneshot::error::TryRecvError::Empty)) | None => None,
            };
            if let Some(reason) = reason {
                self.depart(index, reason);
            }
        }
    }

    fn rollback(&mut self, index: usize) {
        self.departures[index] = None;
        self.bindings[index] = None;
        let _ = self
            .lifecycle
            .depart(SeatId::ALL[index], DepartureReason::TransportLoss);
    }

    fn after_lifecycle_change(&mut self) {
        if matches!(
            self.lifecycle.phase(),
            Phase::Result { .. } | Phase::Interrupted { .. }
        ) {
            self.terminal_at.get_or_insert_with(Instant::now);
        }
        if self.state.is_some() {
            let _ = self.publish_snapshot();
        } else {
            let _ = self.publish_view();
        }
    }

    pub(super) fn tick(&mut self) -> bool {
        match self.lifecycle.phase() {
            Phase::Waiting | Phase::Countdown { .. } => {
                let change = match self.lifecycle.advance_idle_tick() {
                    Ok(change) => change,
                    Err(_) => {
                        self.interrupt(InterruptReason::InternalError);
                        return true;
                    }
                };
                if change == grenadiers_core::room::Change::Started {
                    let Some(state) = initial_state(self.lifecycle.tick()) else {
                        self.interrupt(InterruptReason::InternalError);
                        return true;
                    };
                    self.state = Some(state);
                    self.publish_snapshot()
                } else {
                    true
                }
            }
            Phase::Running => {
                let Some(next_tick) = self.lifecycle.tick().0.checked_add(1).map(Tick) else {
                    self.interrupt(InterruptReason::InternalError);
                    return true;
                };
                let inputs = self.inputs.take(next_tick);
                let report = match self.state.as_mut().expect("running state").step(&inputs) {
                    Ok(report) => report,
                    Err(_) => {
                        self.interrupt(InterruptReason::InternalError);
                        return true;
                    }
                };
                let result = self.state.as_ref().expect("running state").result();
                if self.lifecycle.complete_running_tick(result).is_err()
                    || report.tick != self.lifecycle.tick()
                {
                    self.interrupt(InterruptReason::InternalError);
                    return true;
                }
                self.reports.push(report);
                if result.is_some() {
                    self.terminal_at = Some(Instant::now());
                    self.inputs.clear_pending();
                    self.publish_snapshot()
                } else if self.reports.len() == 3 {
                    self.publish_snapshot()
                } else {
                    true
                }
            }
            Phase::Result { .. } => {
                let report = match self.state.as_mut().expect("result state").step(&[]) {
                    Ok(report) => report,
                    Err(_) => return false,
                };
                if self.lifecycle.advance_idle_tick().is_err()
                    || report.tick != self.lifecycle.tick()
                {
                    return false;
                }
                self.reports.push(report);
                if self.reports.len() == 3 {
                    self.publish_snapshot()
                } else {
                    true
                }
            }
            Phase::Interrupted { .. } => true,
        }
    }

    pub(super) fn interrupt(&mut self, reason: InterruptReason) {
        self.lifecycle.interrupt(reason);
        self.inputs.clear_pending();
        self.terminal_at.get_or_insert_with(Instant::now);
        if self.state.is_some() {
            let _ = self.publish_snapshot();
        } else {
            let _ = self.publish_view();
        }
    }

    fn publish_view(&mut self) -> Option<grenadiers_core::protocol::RoomView> {
        let Some(next) = self.revision.checked_add(1) else {
            self.failed = true;
            return None;
        };
        let Ok(view) = publication::room_view(&self.identity, &self.lifecycle, next) else {
            self.failed = true;
            return None;
        };
        self.revision = next;
        self.notices.send_replace(RoomNotice::View(view.clone()));
        Some(view)
    }

    fn publish_snapshot(&mut self) -> bool {
        let Some(next) = self.revision.checked_add(1) else {
            self.failed = true;
            return false;
        };
        let Ok((_view, encoded)) = publication::snapshot(
            &self.identity,
            &self.lifecycle,
            next,
            self.state.as_ref().expect("snapshot state"),
            &self.reports,
        ) else {
            self.failed = true;
            return false;
        };
        self.revision = next;
        self.reports.clear();
        self.notices.send_replace(RoomNotice::Snapshot(encoded));
        true
    }
}

fn initial_state(tick: Tick) -> Option<State> {
    let arena = fixed_arena();
    let spawns = arena.spawns();
    let actors = [
        ActorInit {
            id: EntityId(1),
            seat: SeatId::One,
            position: Position::center(spawns[0]),
            heading: Direction::East,
            speed: SpeedLevel::Base,
        },
        ActorInit {
            id: EntityId(2),
            seat: SeatId::Two,
            position: Position::center(spawns[1]),
            heading: Direction::West,
            speed: SpeedLevel::Base,
        },
    ];
    State::new_at_tick(RulesConfig::classic_v1(), World::new(arena), actors, tick).ok()
}

fn input_error(error: InputError) -> RoomError {
    match error {
        InputError::RateLimited => RoomError::RateLimited,
        InputError::DuplicateAction => RoomError::DuplicateAction,
        InputError::TickOverflow => RoomError::CapacityReached,
    }
}

#[cfg(test)]
#[path = "runtime_tests.rs"]
mod tests;
