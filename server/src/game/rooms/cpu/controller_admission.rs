use std::sync::mpsc::TryRecvError;

use grenadiers_core::cpu::policy::MAX_HOLD_TICKS;
use grenadiers_core::cpu::survival::{
    CommittedPulse, CommittedRoute, ExecutionCertificate, MAX_PLAN_AGE_TICKS,
};
use grenadiers_core::cpu::{AdmissionContext, DecisionAdmission, WorkBudget};
use grenadiers_core::game::Tick;

use super::{AcceptedPlan, CpuController};

impl CpuController {
    pub(in crate::game::rooms) fn poll(
        &mut self,
        current_tick: Tick,
        command_epoch: u64,
    ) -> Option<AcceptedPlan> {
        if self.escaping || self.committed_pulse.is_some() || self.committed_route.is_some() {
            self.cancel();
            return None;
        }
        let pending = self.pending.as_ref()?;
        let Some(age) = current_tick
            .0
            .checked_sub(pending.decision.observation_tick.0)
        else {
            self.cancel();
            return None;
        };
        if age > MAX_PLAN_AGE_TICKS
            || pending.decision.command_epoch != command_epoch
            || self.world != Some(pending.decision.world)
        {
            self.cancel();
            return None;
        }
        let received = if pending.completed.is_none() {
            match pending.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    return None;
                }
            }
        } else {
            None
        };
        if let Some(result) = received {
            self.pending.as_mut()?.completed = Some(result);
        }
        let result = self.pending.as_ref()?.completed.as_ref()?;
        let manual_admission = result
            .plan
            .execution
            .as_ref()
            .and_then(ExecutionCertificate::manual_execution_tick)
            .and_then(|execution| execution.0.checked_sub(1))
            .map(Tick);
        if let Some(admission) = manual_admission {
            match current_tick.cmp(&admission) {
                std::cmp::Ordering::Less => return None,
                std::cmp::Ordering::Greater => {
                    self.cancel();
                    return None;
                }
                std::cmp::Ordering::Equal => {}
            }
        }
        self.admit_completed(current_tick, command_epoch)
    }

    fn admit_completed(&mut self, current_tick: Tick, command_epoch: u64) -> Option<AcceptedPlan> {
        let pending = self.pending.take()?;
        let result = pending.completed?;
        let ordinary_navigation = result.plan.lease.is_none()
            && result.plan.execution.is_none()
            && result.plan.control.intent.manual.is_none()
            && result.plan.control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop
            && super::cadence::is_clear_for_easy(&result.basis);
        let latest = self.latest_observation.as_ref()?;
        let escape_origin = match (&result.escape_origin, &self.escape_origin) {
            (None, None) => false,
            (Some(result), Some(current)) if result.same_origin(current) => true,
            _ => return None,
        };
        if self.settled_stop_tick.is_none()
            || (result.basis.parts().rules.version == 9
                && result.plan.control.intent.movement
                    != grenadiers_core::cpu::MovementIntent::Stop
                && result.plan.lease.is_none()
                && result.plan.execution.is_none())
            || ((matches!(result.basis.parts().rules.version, 8 | 9)
                || result.plan.execution.is_none())
                && !grenadiers_core::cpu::survival::can_admit_ordinary(&result.basis, latest))
        {
            return None;
        }
        if !(1..=MAX_HOLD_TICKS).contains(&result.plan.control.hold_ticks) {
            return None;
        }
        let grenadiers_core::cpu::survival::SurvivalPlan {
            control,
            lease,
            execution,
        } = result.plan;
        let contact = match result.contact {
            Some(contact)
                if contact.safe_until() > current_tick
                    && contact.observation_fits(latest, &mut WorkBudget::new(65_536)) =>
            {
                Some(contact)
            }
            Some(_) => return None,
            None if (result.basis.parts().rules.version == 8
                || (result.basis.parts().rules.version == 9
                    && result.basis.respawn_lairs().is_some()))
                && (control.intent.manual.is_some()
                    || control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop) =>
            {
                return None;
            }
            None => None,
        };
        let certified_manual = execution
            .as_ref()
            .and_then(ExecutionCertificate::manual_intent);
        let invalid_manual = execution
            .as_ref()
            .is_some_and(|_| certified_manual != control.intent.manual);
        if let (Some(intent), Some(plant)) = (
            control.next_memory.strategic_intent(),
            execution.as_ref().and_then(ExecutionCertificate::plant),
        ) && !grenadiers_core::cpu::survival::strategic_plant_still_targets(
            latest,
            intent,
            plant,
            &mut WorkBudget::new(65_536),
        ) {
            return None;
        }
        if (escape_origin
            && (control.intent.manual.is_some()
                || execution.is_some()
                || (control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop
                    && lease.is_none())))
            || (execution.is_some() && (lease.is_some() || invalid_manual))
        {
            return None;
        }
        let committed_pulse = match lease {
            Some(lease) => Some(
                CommittedPulse::admit(lease, result.basis.clone(), latest, current_tick).ok()?,
            ),
            None => None,
        };
        let committed_route = match execution {
            Some(execution) => Some(
                CommittedRoute::admit_execution(
                    execution,
                    result.basis,
                    latest,
                    current_tick,
                    control.intent.movement,
                )
                .ok()?,
            ),
            None => None,
        };
        let current = AdmissionContext {
            world: self.world?,
            command_epoch,
            observation_tick: pending.decision.observation_tick,
            checked_request_serial: self.request_serial,
        };
        if grenadiers_core::cpu::admit_decision(
            self.admission_memory.as_mut()?,
            current,
            pending.decision,
            &mut WorkBudget::new(1),
        ) != DecisionAdmission::Accepted
        {
            return None;
        }
        Some(AcceptedPlan {
            decision: pending.decision,
            intent: control.intent,
            hold_ticks: control.hold_ticks,
            next_memory: control.next_memory,
            next_rng: result.rng,
            committed_pulse,
            committed_route,
            contact,
            certified_manual,
            escape_origin,
            ordinary_navigation,
        })
    }
}
