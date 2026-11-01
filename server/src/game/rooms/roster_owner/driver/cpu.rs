use grenadiers_core::{
    cpu::{Intent, ManualIntent, MovementIntent, survival::RouteStatus},
    game::{SeatId, Tick},
    protocol::v14,
    room::MatchPhase,
};

use super::{Driver, fatal};
use crate::game::rooms::{
    cpu::{AcceptedPlan, ContactStatus, ContinuationService},
    roster_runtime::{RosterAction, RosterRuntimeError},
};

impl Driver {
    pub(in crate::game::rooms::roster_owner) fn service_cpu(
        &mut self,
    ) -> Result<(), RosterRuntimeError> {
        if self.cpus.is_empty() {
            return Ok(());
        }
        if self.runtime.lifecycle().phase() != MatchPhase::Running {
            for cpu in &mut self.cpus {
                cpu.cancel();
            }
            self.cpu_stop_at = [None; 4];
            self.cpu_stationary = [true; 4];
            return Ok(());
        }
        let count = self.cpus.len();
        let first = self.next_cpu % count;
        self.next_cpu = (first + 1) % count;
        for offset in 0..count {
            let index = (first + offset) % count;
            if self.apply_teleporter(index)? {
                continue;
            }
            if self.apply_continuation(index)? {
                continue;
            }
            self.apply_route(index)?;
            self.apply_stop(index)?;
            let tick = self.runtime.lifecycle().tick();
            let epoch = self.runtime.lifecycle().round_attempt();
            if let Some(plan) = self.cpus[index].poll(tick, epoch) {
                self.apply_plan(index, plan)?;
            }
        }
        Ok(())
    }

    fn apply_continuation(&mut self, index: usize) -> Result<bool, RosterRuntimeError> {
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        match self.cpus[index].service_continuation(tick, epoch) {
            ContinuationService::Idle => Ok(false),
            ContinuationService::Controlled | ContinuationService::Completed => Ok(true),
            ContinuationService::Invalidated => {
                self.fail_continuation(index)?;
                Ok(true)
            }
            ContinuationService::Submit(submission) => {
                let seat = self.cpus[index].seat();
                if submission.admission_tick() != tick {
                    self.fail_continuation(index)?;
                    return Ok(true);
                }
                let direction = movement(submission.movement());
                #[cfg(test)]
                if std::mem::take(&mut self.continuation_rejections[0]) {
                    self.fail_continuation(index)?;
                    return Ok(true);
                }
                match self
                    .runtime
                    .submit_cpu_intent(seat, epoch, RosterAction::Movement(direction))
                {
                    Ok(_) => {}
                    Err(error) if fatal(error) => {
                        self.fail_continuation(index)?;
                        return Err(error);
                    }
                    Err(_) => {
                        self.fail_continuation(index)?;
                        return Ok(true);
                    }
                }
                if !self.cpus[index].commit_continuation(submission) {
                    self.fail_continuation(index)?;
                    return Ok(true);
                }
                self.cpu_stationary[seat.index()] = direction.is_none();
                self.cpu_stop_at[seat.index()] = None;
                if direction.is_some() {
                    self.cpus[index].on_movement_admitted();
                } else {
                    self.cpus[index].on_stop_admitted(tick);
                }
                Ok(true)
            }
        }
    }

    fn fail_continuation(&mut self, index: usize) -> Result<(), RosterRuntimeError> {
        let seat = self.cpus[index].seat();
        let epoch = self.runtime.lifecycle().round_attempt();
        #[cfg(test)]
        let reject_stop = std::mem::take(&mut self.continuation_rejections[1]);
        #[cfg(not(test))]
        let reject_stop = false;
        let result = if reject_stop {
            Err(RosterRuntimeError::InvalidPhase)
        } else {
            self.runtime
                .submit_cpu_intent(seat, epoch, RosterAction::Movement(None))
        };
        self.runtime.stop_seat_inputs(seat);
        self.cpus[index].finish_continuation_fence();
        self.cpu_stationary[seat.index()] = true;
        self.cpu_stop_at[seat.index()] = None;
        match result {
            Err(error) if fatal(error) => Err(error),
            Ok(_) | Err(_) => Ok(()),
        }
    }

    fn apply_plan(&mut self, index: usize, plan: AcceptedPlan) -> Result<(), RosterRuntimeError> {
        if plan.is_refuge_handoff() {
            return self.apply_refuge_handoff(index, plan);
        }
        let seat = self.cpus[index].seat();
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        let movement = movement(plan.intent.movement);
        let has_route = plan.has_route();
        match self
            .runtime
            .submit_cpu_intent(seat, epoch, RosterAction::Movement(movement))
        {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => {
                self.cpus[index].clear_escape_origin();
                return Ok(());
            }
        }
        let manual = self.apply_manual(index, plan.intent.manual)?;
        self.cpu_stationary[seat.index()] = movement.is_none();
        self.cpu_stop_at[seat.index()] = if movement.is_some() && !has_route {
            tick.0.checked_add(u64::from(plan.hold_ticks)).map(Tick)
        } else {
            None
        };
        if movement.is_some() {
            self.cpus[index].on_movement_admitted();
        } else {
            self.cpus[index].on_stop_admitted(tick);
        }
        self.cpus[index].commit(
            plan,
            Intent {
                movement: if let Some(direction) = movement {
                    MovementIntent::Direction(direction)
                } else {
                    MovementIntent::Stop
                },
                manual,
            },
        );
        Ok(())
    }

    fn apply_refuge_handoff(
        &mut self,
        index: usize,
        plan: AcceptedPlan,
    ) -> Result<(), RosterRuntimeError> {
        debug_assert_eq!(plan.intent.movement, MovementIntent::Stop);
        debug_assert_eq!(plan.intent.manual, Some(ManualIntent::Bomb));
        let seat = self.cpus[index].seat();
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        match self
            .runtime
            .submit_cpu_intent(seat, epoch, RosterAction::Movement(None))
        {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => return Ok(()),
        }
        if self.apply_manual(index, Some(ManualIntent::Bomb))? != Some(ManualIntent::Bomb) {
            return Ok(());
        }
        self.cpu_stationary[seat.index()] = true;
        self.cpu_stop_at[seat.index()] = None;
        self.cpus[index].on_stop_admitted(tick);
        self.cpus[index].commit(plan, Intent::STOP.with_manual(ManualIntent::Bomb));
        Ok(())
    }

    fn apply_manual(
        &mut self,
        index: usize,
        manual: Option<ManualIntent>,
    ) -> Result<Option<ManualIntent>, RosterRuntimeError> {
        let Some(manual) = manual else {
            return Ok(None);
        };
        let seat = self.cpus[index].seat();
        let Some(id) = self.runtime.next_cpu_manual_action(seat) else {
            return Ok(None);
        };
        let action = match manual {
            ManualIntent::Bomb => RosterAction::Bomb(id),
            ManualIntent::Equipment => RosterAction::Equipment(id),
        };
        match self
            .runtime
            .submit_cpu_intent(seat, self.runtime.lifecycle().round_attempt(), action)
        {
            Ok(_) => Ok(Some(manual)),
            Err(error) if fatal(error) => Err(error),
            Err(_) => Ok(None),
        }
    }

    fn apply_escape(
        &mut self,
        index: usize,
        plan: grenadiers_core::cpu::policy::ControlPlan,
    ) -> Result<(), RosterRuntimeError> {
        let seat = self.cpus[index].seat();
        let tick = self.runtime.lifecycle().tick();
        let direction = movement(plan.intent.movement);
        match self.runtime.submit_cpu_intent(
            seat,
            self.runtime.lifecycle().round_attempt(),
            RosterAction::Movement(direction),
        ) {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => return Ok(()),
        }
        self.cpu_stationary[seat.index()] = direction.is_none();
        self.cpu_stop_at[seat.index()] = direction
            .is_some()
            .then(|| tick.0.checked_add(u64::from(plan.hold_ticks)).map(Tick))
            .flatten();
        if direction.is_some() {
            self.cpus[index].on_movement_admitted();
        } else {
            self.cpus[index].on_stop_admitted(tick);
        }
        self.cpus[index].commit_escape(plan);
        Ok(())
    }

    fn apply_stop(&mut self, index: usize) -> Result<(), RosterRuntimeError> {
        let seat = self.cpus[index].seat();
        let tick = self.runtime.lifecycle().tick();
        if !self.cpu_stop_at[seat.index()].is_some_and(|deadline| tick >= deadline) {
            return Ok(());
        }
        match self.runtime.submit_cpu_intent(
            seat,
            self.runtime.lifecycle().round_attempt(),
            RosterAction::Movement(None),
        ) {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => {
                self.cpus[index].clear_escape_origin();
                return Ok(());
            }
        }
        self.cpu_stop_at[seat.index()] = None;
        self.cpu_stationary[seat.index()] = true;
        self.cpus[index].on_stop_admitted(tick);
        Ok(())
    }

    fn apply_route(&mut self, index: usize) -> Result<(), RosterRuntimeError> {
        let seat = self.cpus[index].seat();
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        let Some(status) = self.cpus[index].poll_route_command(tick, epoch) else {
            return Ok(());
        };
        let RouteStatus::Submit(command) = status else {
            if status == RouteStatus::Invalidated {
                self.cpu_stop_at[seat.index()] = Some(tick);
            }
            return Ok(());
        };
        let direction = movement(command.movement());
        match self
            .runtime
            .submit_cpu_intent(seat, epoch, RosterAction::Movement(direction))
        {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => {
                self.cpus[index].invalidate_route();
                self.cpu_stop_at[seat.index()] = Some(tick);
                return Ok(());
            }
        }
        self.cpu_stationary[seat.index()] = direction.is_none();
        self.cpu_stop_at[seat.index()] = None;
        if direction.is_some() {
            self.cpus[index].on_movement_admitted();
        } else {
            self.cpus[index].on_stop_admitted(tick);
        }
        if !self.cpus[index].commit_route_command(command) {
            self.cpus[index].invalidate_route();
            self.cpu_stop_at[seat.index()] = Some(tick);
        }
        Ok(())
    }
}

const fn movement(intent: MovementIntent) -> Option<grenadiers_core::game::Direction> {
    match intent {
        MovementIntent::Direction(direction) => Some(direction),
        MovementIntent::Stop => None,
    }
}

const fn wire_seat(seat: SeatId) -> v14::Seat {
    match seat {
        SeatId::One => v14::Seat::One,
        SeatId::Two => v14::Seat::Two,
        SeatId::Three => v14::Seat::Three,
        SeatId::Four => v14::Seat::Four,
    }
}

#[cfg(test)]
#[path = "../refuge_handoff_observation_support.rs"]
mod refuge_handoff_observation_support;

#[cfg(test)]
#[path = "../refuge_handoff_owner_support.rs"]
mod refuge_handoff_owner_support;

#[cfg(test)]
#[path = "../refuge_handoff_owner_tests.rs"]
mod refuge_handoff_owner_tests;

#[path = "../driver_cpu_observation.rs"]
mod observation;

#[path = "../driver_cpu_teleporter.rs"]
mod teleporter;
