use grenadiers_core::{
    cpu::{
        Intent, ManualIntent, MovementIntent,
        policy::EscapeExecution,
        survival::{PulseStatus, RouteStatus},
    },
    game::{SeatId, Tick},
    room::{MatchPhase, reconnect::ReservationState},
};

use super::{Driver, fatal};
use crate::game::rooms::match_runtime::{MatchAction, MatchRuntimeError};

impl Driver {
    pub(in crate::game::rooms) fn is_empty(&self) -> bool {
        if let Some(cpu_seat) = self.cpu.as_ref().map(|cpu| cpu.seat()) {
            return SeatId::DUEL
                .into_iter()
                .filter(|seat| *seat != cpu_seat)
                .all(|seat| {
                    matches!(
                        self.runtime.lifecycle().reconnect_grace().seat(seat),
                        Some(ReservationState::Vacant | ReservationState::Expired)
                    )
                });
        }
        self.seats.iter().all(Option::is_none)
    }

    pub(in crate::game::rooms) fn poll_cpu_result(&mut self) -> Result<(), MatchRuntimeError> {
        if self.runtime.lifecycle().phase() != MatchPhase::Running {
            return Ok(());
        }
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        let Some(plan) = self.cpu.as_mut().and_then(|cpu| cpu.poll(tick, epoch)) else {
            return Ok(());
        };
        self.apply_cpu_plan(plan)
    }

    fn apply_cpu_plan(
        &mut self,
        plan: crate::game::rooms::cpu::AcceptedPlan,
    ) -> Result<(), MatchRuntimeError> {
        let tick = self.runtime.lifecycle().tick();
        let epoch = self.runtime.lifecycle().round_attempt();
        let cpu_seat = self
            .cpu
            .as_ref()
            .expect("accepted plan has controller")
            .seat();
        let movement = match plan.intent.movement {
            MovementIntent::Direction(direction) => Some(direction),
            MovementIntent::Stop => None,
        };
        let has_route = plan.has_route();
        match self
            .runtime
            .submit_cpu_intent(cpu_seat, epoch, MatchAction::Movement(movement))
        {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) if has_route => {
                self.cpu
                    .as_mut()
                    .expect("accepted plan has controller")
                    .clear_escape_origin();
                self.cpu_stop_at = Some(tick);
                return self.apply_cpu_stop();
            }
            Err(_) => {
                self.cpu
                    .as_mut()
                    .expect("accepted plan has controller")
                    .clear_escape_origin();
                return Ok(());
            }
        }
        let manual = self.apply_cpu_manual(epoch, plan.intent.manual)?;
        let actual = Intent {
            movement: plan.intent.movement,
            manual,
        };
        self.cpu_stationary = movement.is_none();
        self.cpu_stop_at = if movement.is_some() && !has_route {
            tick.0.checked_add(u64::from(plan.hold_ticks)).map(Tick)
        } else {
            None
        };
        let cpu = self.cpu.as_mut().expect("accepted plan has controller");
        if movement.is_some() {
            cpu.on_movement_admitted();
        } else {
            cpu.on_stop_admitted(tick);
        }
        cpu.commit(plan, actual);
        Ok(())
    }

    fn apply_escape_plan(
        &mut self,
        plan: grenadiers_core::cpu::policy::ControlPlan,
    ) -> Result<(), MatchRuntimeError> {
        let tick = self.runtime.lifecycle().tick();
        let cpu_seat = self
            .cpu
            .as_ref()
            .expect("escape plan has controller")
            .seat();
        let movement = match plan.intent.movement {
            MovementIntent::Direction(direction) => Some(direction),
            MovementIntent::Stop => None,
        };
        match self.runtime.submit_cpu_intent(
            cpu_seat,
            self.runtime.lifecycle().round_attempt(),
            MatchAction::Movement(movement),
        ) {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => return Ok(()),
        }
        self.cpu_stationary = movement.is_none();
        self.cpu_stop_at = movement
            .is_some()
            .then(|| tick.0.checked_add(u64::from(plan.hold_ticks)).map(Tick))
            .flatten();
        let cpu = self.cpu.as_mut().expect("escape plan has controller");
        if movement.is_some() {
            cpu.on_movement_admitted();
        } else {
            cpu.on_stop_admitted(tick);
        }
        cpu.commit_escape(plan);
        Ok(())
    }

    fn apply_cpu_manual(
        &mut self,
        epoch: u64,
        manual: Option<ManualIntent>,
    ) -> Result<Option<ManualIntent>, MatchRuntimeError> {
        let Some(manual) = manual else {
            return Ok(None);
        };
        let cpu_seat = self
            .cpu
            .as_ref()
            .expect("manual plan has controller")
            .seat();
        let Some(id) = self.runtime.next_cpu_manual_action(cpu_seat) else {
            return Ok(None);
        };
        let action = match manual {
            ManualIntent::Bomb => MatchAction::Bomb(id),
            ManualIntent::Equipment => MatchAction::Equipment(id),
        };
        match self.runtime.submit_cpu_intent(cpu_seat, epoch, action) {
            Ok(_) => Ok(Some(manual)),
            Err(error) if fatal(error) => Err(error),
            Err(_) => Ok(None),
        }
    }

    pub(super) fn apply_cpu_stop(&mut self) -> Result<(), MatchRuntimeError> {
        let tick = self.runtime.lifecycle().tick();
        if !self.cpu_stop_at.is_some_and(|deadline| tick >= deadline) {
            return Ok(());
        }
        let Some(cpu_seat) = self.cpu.as_ref().map(|cpu| cpu.seat()) else {
            self.cpu_stop_at = None;
            return Ok(());
        };
        if self.runtime.lifecycle().phase() == MatchPhase::Running {
            match self.runtime.submit_cpu_intent(
                cpu_seat,
                self.runtime.lifecycle().round_attempt(),
                MatchAction::Movement(None),
            ) {
                Ok(_) => {}
                Err(error) if fatal(error) => return Err(error),
                Err(_) => {
                    if let Some(cpu) = self.cpu.as_mut() {
                        cpu.clear_escape_origin();
                    }
                    return Ok(());
                }
            }
        }
        self.cpu_stop_at = None;
        self.cpu_stationary = true;
        if let Some(cpu) = self.cpu.as_mut() {
            cpu.on_stop_admitted(tick);
        }
        Ok(())
    }

    pub(super) fn consider_cpu_observation(&mut self, notice: &crate::game::rooms::MatchNotice) {
        let Some(cpu) = self.cpu.as_mut() else {
            return;
        };
        if self.runtime.lifecycle().phase() != MatchPhase::Running {
            cpu.cancel();
            cpu.clear_escape_origin();
            if cpu.has_committed_route() {
                cpu.invalidate_route();
            }
            self.cpu_stop_at = None;
            self.cpu_stationary = true;
            return;
        }
        let Some(observation) = notice.cpu_observation.as_ref() else {
            return;
        };
        cpu.validate_escape_epoch(self.runtime.lifecycle().round_attempt());
        if !cpu.accept_observation(observation) {
            return;
        }
        let route = cpu.observe_route(observation);
        if route == Some(RouteStatus::Invalidated) {
            self.cpu_stop_at = Some(self.runtime.lifecycle().tick());
            let _ = self.apply_cpu_stop();
            return;
        }
        if matches!(
            route,
            Some(RouteStatus::Continue | RouteStatus::Wait | RouteStatus::Submit(_))
        ) {
            return;
        }
        let pulse = cpu.validate_pulse(observation, observation.parts().tick);
        if pulse == Some(PulseStatus::Invalidated) && !self.cpu_stationary {
            self.cpu_stop_at = Some(self.runtime.lifecycle().tick());
            let _ = self.apply_cpu_stop();
            return;
        }
        if matches!(pulse, Some(PulseStatus::Continue | PulseStatus::Wait)) {
            return;
        }
        if cpu.is_escaping() {
            let execution = cpu.follow_escape(observation).ok();
            match execution {
                Some(EscapeExecution::Continue(plan) | EscapeExecution::Wait(plan))
                    if self.cpu_stationary =>
                {
                    let _ = self.apply_escape_plan(plan);
                    return;
                }
                Some(EscapeExecution::Continue(_) | EscapeExecution::Wait(_)) => return,
                Some(EscapeExecution::Invalidated) if !self.cpu_stationary => {
                    self.cpu_stop_at = Some(self.runtime.lifecycle().tick());
                    let _ = self.apply_cpu_stop();
                    return;
                }
                Some(EscapeExecution::Completed | EscapeExecution::Invalidated) | None => {}
            }
        }
        if self.cpu_stationary && cpu.needs_stop_confirmation() {
            self.cpu_stop_at = Some(self.runtime.lifecycle().tick());
            let _ = self.apply_cpu_stop();
            return;
        }
        if self.cpu_stationary && !cpu.has_pending() {
            let _ = cpu.submit(
                observation.clone(),
                self.runtime.lifecycle().round_attempt(),
            );
        }
    }

    pub(super) fn reconcile_cpu_lifecycle(&mut self) -> Result<(), MatchRuntimeError> {
        if self.cpu.is_none() {
            return Ok(());
        }
        let cpu_seat = self
            .cpu
            .as_ref()
            .expect("CPU lifecycle has controller")
            .seat();
        let phase = self.runtime.lifecycle().phase();
        if phase == MatchPhase::Waiting
            && !self.runtime.lifecycle().seats()[cpu_seat.index()].ready()
        {
            if let Some(notice) = self.runtime.cpu_ready(cpu_seat)? {
                self.publish(notice);
            }
            return Ok(());
        }
        let MatchPhase::MatchResult { rematch_at, .. } = phase else {
            return Ok(());
        };
        let human_connected = SeatId::DUEL
            .into_iter()
            .filter(|seat| *seat != cpu_seat)
            .any(|seat| {
                matches!(
                    self.runtime.lifecycle().reconnect_grace().seat(seat),
                    Some(ReservationState::Connected)
                )
            });
        let cpu_voted = self.runtime.lifecycle().series().rematch_votes()[cpu_seat.index()];
        if human_connected
            && !cpu_voted
            && self.runtime.lifecycle().tick() >= rematch_at
            && let Some(notice) = self.runtime.cpu_rematch(cpu_seat)?
        {
            self.publish(notice);
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "cpu_runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "cpu_runtime_cpu2_tests.rs"]
mod cpu2_tests;

#[cfg(test)]
#[path = "cpu_runtime_route_tests.rs"]
mod route_tests;

#[cfg(test)]
#[path = "cpu_runtime_plant_tests.rs"]
mod plant_tests;

#[cfg(test)]
#[path = "cpu_runtime_weapon_tests.rs"]
mod weapon_tests;

#[cfg(test)]
#[path = "cpu_runtime_wall_pass_tests.rs"]
mod wall_pass_tests;

#[cfg(test)]
#[path = "cpu_bracelet_tactic_tests.rs"]
mod bracelet_tactic_tests;

#[cfg(test)]
#[path = "cpu_runtime_punch_tests.rs"]
mod punch_tests;

#[cfg(test)]
#[path = "cpu_runtime_equipment_goal_tests.rs"]
mod equipment_goal_tests;

#[cfg(test)]
#[path = "cpu_runtime_responsiveness_tests.rs"]
mod responsiveness_tests;

#[cfg(test)]
#[path = "cpu_evaluation_support.rs"]
pub(super) mod evaluation_support;

#[cfg(test)]
#[path = "cpu_evaluation_tests.rs"]
mod evaluation_tests;

#[cfg(test)]
#[path = "cpu_escape_recovery_tests.rs"]
mod escape_recovery_tests;

#[cfg(test)]
#[path = "cpu_late_escape_recovery_tests.rs"]
mod late_escape_recovery_tests;

#[cfg(test)]
#[path = "cpu_pressure_progress_tests.rs"]
mod pressure_progress_tests;

#[cfg(test)]
#[path = "cpu_turn_alignment_tests.rs"]
mod turn_alignment_tests;

#[cfg(test)]
#[path = "cpu_runtime_cadence_tests.rs"]
mod cadence_tests;
