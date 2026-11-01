//! Inactive one-move teleporter executor with an explicit physical Stop fence.

use std::sync::Arc;

use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::RouteFailure;
use crate::cpu::{DecisionId, ManualIntent, MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::teleporter_execution_types::{
    CommittedTeleporterStep, EpisodeToken, PreparedTeleporterStep, TeleporterOutcome,
    TeleporterStepCommand, TeleporterStepStatus, command,
};
use super::teleporter_recovery_types::TeleporterStepPlan;

pub const TELEPORTER_ADMISSION_ALLOWANCE: u32 = 8_192;

impl PreparedTeleporterStep {
    #[allow(clippy::too_many_arguments)]
    pub fn admit(
        plan: TeleporterStepPlan,
        basis: Observation,
        current: &Observation,
        actual_settled_stop_tick: Tick,
        decision: DecisionId,
        admission_tick: Tick,
        work: &mut WorkBudget,
    ) -> Result<Self, RouteFailure> {
        let mut bounded = WorkBudget::new(work.remaining().min(TELEPORTER_ADMISSION_ALLOWANCE));
        let result = (|| {
            let first_update = add_tick(admission_tick, 1)?;
            let key = MemoryKey {
                world: basis.parts().world,
                cpu_seat: basis.parts().recipient,
            };
            if decision.world != basis.parts().world
                || decision.observation_tick != basis.parts().tick
                || current.parts().tick != admission_tick
                || plan.step.key() != key
                || plan.next_memory.key() != key
                || actual_settled_stop_tick != plan.step.settled_stop_tick()
                || actual_settled_stop_tick > plan.step.basis_tick()
                || !plan.step.start_window().contains(first_update)
                || !super::teleporter_execution_compat::admission_fits(
                    &plan.step,
                    &basis,
                    current,
                    admission_tick,
                    &mut bounded,
                )?
            {
                return Err(RouteFailure::IdentityMismatch);
            }
            Ok(Self {
                token: Arc::new(EpisodeToken),
                step: Box::new(plan.step),
                next_memory: plan.next_memory,
                decision,
                admission_tick,
                command_issued: false,
            })
        })();
        work.try_take(bounded.spent())
            .map_err(|_| RouteFailure::WorkExhausted)?;
        result
    }

    #[must_use]
    pub const fn movement(&self) -> MovementIntent {
        self.step.movement()
    }
    #[must_use]
    pub const fn hold_ticks(&self) -> u8 {
        1
    }
    #[must_use]
    pub const fn manual(&self) -> Option<ManualIntent> {
        None
    }

    pub fn command(&mut self) -> Result<TeleporterStepCommand, RouteFailure> {
        if self.command_issued {
            return Err(RouteFailure::IdentityMismatch);
        }
        self.command_issued = true;
        Ok(self.expected_command())
    }

    pub fn commit(
        self,
        accepted: TeleporterStepCommand,
    ) -> Result<CommittedTeleporterStep, RouteFailure> {
        if !self.command_issued || !accepted.matches(&self.expected_command()) {
            return Err(RouteFailure::IdentityMismatch);
        }
        Ok(CommittedTeleporterStep {
            token: self.token,
            step: self.step,
            next_memory: self.next_memory,
            decision: self.decision,
            admission_tick: self.admission_tick,
            stop_issued: false,
            stop_recorded: false,
            completed: None,
            revoked: false,
        })
    }

    fn expected_command(&self) -> TeleporterStepCommand {
        command(
            &self.token,
            self.step.key(),
            self.decision.command_epoch,
            self.step.movement(),
            self.admission_tick,
            0,
        )
    }
}

impl CommittedTeleporterStep {
    pub fn poll_command(
        &mut self,
        current_tick: Tick,
        key: MemoryKey,
        command_epoch: u64,
    ) -> TeleporterStepStatus {
        if self.revoked || key != self.step.key() || command_epoch != self.decision.command_epoch {
            self.revoked = true;
            return TeleporterStepStatus::Invalidated;
        }
        if let Some(outcome) = self.completed {
            return TeleporterStepStatus::Completed(outcome);
        }
        if current_tick >= self.step.safe_until() {
            self.revoked = true;
            return TeleporterStepStatus::Invalidated;
        }
        let Ok(stop_tick) = add_tick(self.admission_tick, 1) else {
            self.revoked = true;
            return TeleporterStepStatus::Invalidated;
        };
        match current_tick.cmp(&stop_tick) {
            std::cmp::Ordering::Less => TeleporterStepStatus::Moving,
            std::cmp::Ordering::Equal if !self.stop_issued => {
                self.stop_issued = true;
                TeleporterStepStatus::Submit(command(
                    &self.token,
                    self.step.key(),
                    self.decision.command_epoch,
                    MovementIntent::Stop,
                    stop_tick,
                    1,
                ))
            }
            std::cmp::Ordering::Equal if self.stop_recorded => {
                TeleporterStepStatus::WaitForStoppedObservation
            }
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater if !self.stop_recorded => {
                self.revoked = true;
                TeleporterStepStatus::Invalidated
            }
            std::cmp::Ordering::Equal => TeleporterStepStatus::WaitForStoppedObservation,
            std::cmp::Ordering::Greater => TeleporterStepStatus::WaitForStoppedObservation,
        }
    }

    /// Records the owner's unconditional input clear at the Stop admission.
    pub fn record_stop(&mut self, submitted: TeleporterStepCommand) -> bool {
        let Ok(stop_tick) = add_tick(self.admission_tick, 1) else {
            self.revoked = true;
            return false;
        };
        let expected = command(
            &self.token,
            self.step.key(),
            self.decision.command_epoch,
            MovementIntent::Stop,
            stop_tick,
            1,
        );
        if self.revoked || !self.stop_issued || self.stop_recorded || !submitted.matches(&expected)
        {
            self.revoked = true;
            return false;
        }
        self.stop_recorded = true;
        true
    }

    pub fn observe_stopped(
        &mut self,
        current: &Observation,
        actual_settled_stop_tick: Tick,
        work: &mut WorkBudget,
    ) -> TeleporterStepStatus {
        let mut bounded = WorkBudget::new(work.remaining().min(TELEPORTER_ADMISSION_ALLOWANCE));
        let result = self.observe_stopped_inner(current, actual_settled_stop_tick, &mut bounded);
        if work.try_take(bounded.spent()).is_err() {
            self.revoked = true;
            return TeleporterStepStatus::Invalidated;
        }
        result
    }

    fn observe_stopped_inner(
        &mut self,
        current: &Observation,
        actual_settled_stop_tick: Tick,
        work: &mut WorkBudget,
    ) -> TeleporterStepStatus {
        let Ok(physical_stop) = add_tick(self.admission_tick, 2) else {
            self.revoked = true;
            return TeleporterStepStatus::Invalidated;
        };
        let movement_tick = Tick(self.admission_tick.0 + 1);
        if self.revoked
            || !self.stop_recorded
            || current.parts().tick < physical_stop
            || current.parts().tick >= self.step.safe_until()
            || actual_settled_stop_tick != physical_stop
            || !super::teleporter_execution_compat::stopped_fits(
                &self.step,
                current,
                movement_tick,
                work,
            )
            .unwrap_or(false)
        {
            self.revoked = true;
            return TeleporterStepStatus::Invalidated;
        }
        let outcome = TeleporterOutcome {
            swapped: super::compatibility::actor(current)
                .is_some_and(|actor| actor.last_teleport_tick == Some(movement_tick)),
            consumed: !current
                .parts()
                .pickups
                .iter()
                .any(|pickup| pickup.cell == self.step.target()),
        };
        self.completed = Some(outcome);
        TeleporterStepStatus::Completed(outcome)
    }

    #[must_use]
    pub const fn next_memory(&self) -> Option<UtilityMemory> {
        if self.completed.is_some() && !self.revoked {
            Some(self.next_memory)
        } else {
            None
        }
    }

    pub fn revoke(&mut self, key: MemoryKey, command_epoch: u64) -> bool {
        if key != self.step.key() || command_epoch != self.decision.command_epoch {
            return false;
        }
        self.revoked = true;
        true
    }
}

fn add_tick(tick: Tick, amount: u64) -> Result<Tick, RouteFailure> {
    tick.0
        .checked_add(amount)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)
}
