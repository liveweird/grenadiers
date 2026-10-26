use grenadiers_core::{
    cpu::{Intent, ManualIntent, MovementIntent, policy::EscapeExecution, survival::PulseStatus},
    game::{SeatId, Tick},
    room::{MatchPhase, reconnect::ReservationState},
};

use super::{Driver, fatal};
use crate::game::rooms::match_runtime::{MatchAction, MatchRuntimeError};

impl Driver {
    pub(in crate::game::rooms) fn is_empty(&self) -> bool {
        if self.cpu.is_some() {
            return matches!(
                self.runtime.lifecycle().reconnect_grace().seat(SeatId::One),
                ReservationState::Vacant | ReservationState::Expired
            );
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
        let movement = match plan.intent.movement {
            MovementIntent::Direction(direction) => Some(direction),
            MovementIntent::Stop => None,
        };
        match self
            .runtime
            .action(SeatId::Two, epoch, MatchAction::Movement(movement))
        {
            Ok(_) => {}
            Err(error) if fatal(error) => return Err(error),
            Err(_) => return Ok(()),
        }
        let manual = self.apply_cpu_manual(epoch, plan.intent.manual)?;
        let actual = Intent {
            movement: plan.intent.movement,
            manual,
        };
        self.cpu_stationary = movement.is_none();
        self.cpu_stop_at = if movement.is_some() {
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
        let movement = match plan.intent.movement {
            MovementIntent::Direction(direction) => Some(direction),
            MovementIntent::Stop => None,
        };
        match self.runtime.action(
            SeatId::Two,
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
        let Some(id) = self.runtime.next_cpu_manual_action() else {
            return Ok(None);
        };
        let action = match manual {
            ManualIntent::Bomb => MatchAction::Bomb(id),
            ManualIntent::Equipment => MatchAction::Equipment(id),
        };
        match self.runtime.action(SeatId::Two, epoch, action) {
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
        if self.runtime.lifecycle().phase() == MatchPhase::Running {
            match self.runtime.action(
                SeatId::Two,
                self.runtime.lifecycle().round_attempt(),
                MatchAction::Movement(None),
            ) {
                Ok(_) => {}
                Err(error) if fatal(error) => return Err(error),
                Err(_) => return Ok(()),
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
            self.cpu_stop_at = None;
            self.cpu_stationary = true;
            return;
        }
        let Some(observation) = notice.cpu_observation.as_ref() else {
            return;
        };
        if !cpu.accept_observation(observation) {
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
        let phase = self.runtime.lifecycle().phase();
        if phase == MatchPhase::Waiting
            && !self.runtime.lifecycle().seats()[SeatId::Two.index()].ready()
        {
            if let Some(notice) = self.runtime.cpu_ready()? {
                self.publish(notice);
            }
            return Ok(());
        }
        let MatchPhase::MatchResult { rematch_at, .. } = phase else {
            return Ok(());
        };
        let human_connected = matches!(
            self.runtime.lifecycle().reconnect_grace().seat(SeatId::One),
            ReservationState::Connected
        );
        let cpu_voted = self.runtime.lifecycle().series().rematch_votes()[SeatId::Two.index()];
        if human_connected
            && !cpu_voted
            && self.runtime.lifecycle().tick() >= rematch_at
            && let Some(notice) = self.runtime.cpu_rematch()?
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
