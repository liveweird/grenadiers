use grenadiers_core::{
    cpu::{
        Intent, ManualIntent, MovementIntent, WorldId,
        policy::EscapeExecution,
        survival::{PulseStatus, RouteStatus},
    },
    game::{SeatId, Tick},
    protocol::{v14, v15},
    room::MatchPhase,
};

use super::{Driver, fatal};
use crate::game::rooms::{
    cpu::AcceptedPlan,
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

    fn apply_plan(&mut self, index: usize, plan: AcceptedPlan) -> Result<(), RosterRuntimeError> {
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

    pub(super) fn observe_cpus(
        &mut self,
        notice: &crate::game::rooms::roster_runtime::RosterNotice,
    ) {
        if self.runtime.lifecycle().phase() != MatchPhase::Running {
            for cpu in &mut self.cpus {
                cpu.cancel();
            }
            return;
        }
        let physical_round = self.runtime.lifecycle().round_attempt();
        let room_instance = self.runtime.identity().registry_serial;
        let world = WorldId {
            room_instance,
            physical_round,
        };
        let observations: Option<Vec<_>> =
            if let Some(publication) = notice.environment_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v15::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else if let Some(publication) = notice.legacy_cpu_publication() {
                self.cpus
                    .iter()
                    .map(|cpu| {
                        v14::observe_cpu_recipient(publication, wire_seat(cpu.seat()), world).ok()
                    })
                    .collect()
            } else {
                return;
            };
        let Some(observations) = observations else {
            return;
        };
        let epoch = self.runtime.lifecycle().round_attempt();
        for (index, observation) in observations.iter().enumerate() {
            let seat = self.cpus[index].seat();
            self.cpus[index].validate_escape_epoch(epoch);
            if !self.cpus[index].accept_observation(observation) {
                continue;
            }
            let route = self.cpus[index].observe_route(observation);
            if route == Some(RouteStatus::Invalidated) {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                continue;
            }
            if matches!(
                route,
                Some(RouteStatus::Continue | RouteStatus::Wait | RouteStatus::Submit(_))
            ) {
                continue;
            }
            let pulse = self.cpus[index].validate_pulse(observation, observation.parts().tick);
            if pulse == Some(PulseStatus::Invalidated) && !self.cpu_stationary[seat.index()] {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                continue;
            }
            if matches!(pulse, Some(PulseStatus::Continue | PulseStatus::Wait)) {
                continue;
            }
            if self.cpus[index].is_escaping() {
                match self.cpus[index].follow_escape(observation).ok() {
                    Some(EscapeExecution::Continue(plan) | EscapeExecution::Wait(plan))
                        if self.cpu_stationary[seat.index()] =>
                    {
                        let _ = self.apply_escape(index, plan);
                        continue;
                    }
                    Some(EscapeExecution::Continue(_) | EscapeExecution::Wait(_)) => continue,
                    Some(EscapeExecution::Invalidated) if !self.cpu_stationary[seat.index()] => {
                        self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
                        continue;
                    }
                    Some(EscapeExecution::Completed | EscapeExecution::Invalidated) | None => {}
                }
            }
            if self.cpu_stationary[seat.index()] && self.cpus[index].needs_stop_confirmation() {
                self.cpu_stop_at[seat.index()] = Some(self.runtime.lifecycle().tick());
            } else if self.cpu_stationary[seat.index()] && !self.cpus[index].has_pending() {
                let _ = self.cpus[index].submit(observation.clone(), epoch);
            }
        }
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
