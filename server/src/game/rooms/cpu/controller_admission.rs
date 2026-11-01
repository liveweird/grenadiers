use std::sync::mpsc::TryRecvError;

use grenadiers_core::cpu::policy::MAX_HOLD_TICKS;
use grenadiers_core::cpu::survival::{
    CommittedPulse, CommittedRoute, ExecutionCertificate, MAX_PLAN_AGE_TICKS,
};
use grenadiers_core::cpu::{AdmissionContext, DecisionAdmission, WorkBudget};
use grenadiers_core::game::Tick;

use super::super::worker::WorkerPlan;
use super::{AcceptedPlan, CpuController};

#[cfg(test)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::game::rooms) struct AdmissionTrace {
    pub expired_or_wrong_decision: u32,
    pub missed_manual_tick: u32,
    pub stale_scene: u32,
    pub invalid_hold: u32,
    pub contact: u32,
    pub strategic_plant: u32,
    pub invalid_shape: u32,
    pub pulse: u32,
    pub route: u32,
    pub duplicate_decision: u32,
    pub refuge_handoff: u32,
}

#[derive(Clone, Copy)]
enum AdmissionRejection {
    ExpiredOrWrongDecision,
    MissedManualTick,
    StaleScene,
    InvalidHold,
    Contact,
    StrategicPlant,
    InvalidShape,
    Pulse,
    Route,
    DuplicateDecision,
    RefugeHandoff,
}

impl CpuController {
    pub(super) fn admit_continuation_decision(
        &mut self,
        decision: grenadiers_core::cpu::DecisionId,
        command_epoch: u64,
    ) -> bool {
        let Some(world) = self.world else {
            return false;
        };
        let current = AdmissionContext {
            world,
            command_epoch,
            observation_tick: decision.observation_tick,
            checked_request_serial: self.request_serial,
        };
        grenadiers_core::cpu::admit_decision(
            self.admission_memory
                .as_mut()
                .expect("world admission memory initialized"),
            current,
            decision,
            &mut WorkBudget::new(1),
        ) == DecisionAdmission::Accepted
    }

    pub(in crate::game::rooms) fn poll(
        &mut self,
        current_tick: Tick,
        command_epoch: u64,
    ) -> Option<AcceptedPlan> {
        let refuge_pending = self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.refuge_handoff.is_some());
        if self.escaping
            || self.committed_pulse.is_some()
            || (self.committed_route.is_some() && !refuge_pending)
            || self.committed_teleporter.is_some()
        {
            self.cancel();
            return None;
        }
        let pending = self.pending.as_ref()?;
        let Some(age) = current_tick
            .0
            .checked_sub(pending.decision.observation_tick.0)
        else {
            self.cancel();
            return self.reject(AdmissionRejection::ExpiredOrWrongDecision);
        };
        if age > MAX_PLAN_AGE_TICKS
            || pending.decision.command_epoch != command_epoch
            || self.world != Some(pending.decision.world)
        {
            self.cancel();
            return self.reject(AdmissionRejection::ExpiredOrWrongDecision);
        }
        let received = if pending.completed.is_none() {
            match pending.receiver.try_recv() {
                Ok(result) => Some(result),
                Err(TryRecvError::Empty) => return None,
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    return self.reject(AdmissionRejection::ExpiredOrWrongDecision);
                }
            }
        } else {
            None
        };
        if let Some(result) = received {
            self.pending.as_mut()?.completed = Some(result);
        }
        let result = self.pending.as_ref()?.completed.as_ref()?;
        let WorkerPlan::Ordinary { plan, .. } = &result.outcome else {
            return None;
        };
        let manual_admission = plan
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
                    return self.reject(AdmissionRejection::MissedManualTick);
                }
                std::cmp::Ordering::Equal => {}
            }
        }
        self.admit_completed(current_tick, command_epoch)
    }

    fn admit_completed(&mut self, current_tick: Tick, command_epoch: u64) -> Option<AcceptedPlan> {
        let pending = self.pending.take()?;
        let result = pending.completed?;
        let WorkerPlan::Ordinary { plan, contact } = result.outcome else {
            return self.reject(AdmissionRejection::InvalidShape);
        };
        let ordinary_navigation = plan.lease.is_none()
            && plan.execution.is_none()
            && plan.control.intent.manual.is_none()
            && plan.control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop
            && super::cadence::is_clear_for_easy(&result.basis);
        let latest = self.latest_observation.as_ref()?;
        let refuge_handoff = pending.refuge_handoff.as_ref();
        if refuge_handoff.is_some_and(|handoff| {
            result.escape_origin.is_some()
                || plan.lease.is_some()
                || plan.control.intent
                    != grenadiers_core::cpu::Intent {
                        movement: grenadiers_core::cpu::MovementIntent::Stop,
                        manual: Some(grenadiers_core::cpu::ManualIntent::Bomb),
                    }
                || plan
                    .execution
                    .as_ref()
                    .is_none_or(|execution| !handoff.accepts_replacement(execution))
                || !self.refuge_handoff_still_installed(handoff, latest)
        }) {
            return self.reject(AdmissionRejection::RefugeHandoff);
        }
        let escape_origin = match (&result.escape_origin, &self.escape_origin) {
            (None, None) => false,
            (Some(result), Some(current)) if result.same_origin(current) => true,
            _ => return self.reject(AdmissionRejection::StaleScene),
        };
        if self.settled_stop_tick.is_none()
            || (matches!(result.basis.parts().rules.version, 9 | 10)
                && plan.control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop
                && plan.lease.is_none()
                && plan.execution.is_none())
            || ((matches!(result.basis.parts().rules.version, 8 | 9) || plan.execution.is_none())
                && !grenadiers_core::cpu::survival::can_admit_ordinary(&result.basis, latest))
        {
            return self.reject(AdmissionRejection::StaleScene);
        }
        if !(1..=MAX_HOLD_TICKS).contains(&plan.control.hold_ticks) {
            return self.reject(AdmissionRejection::InvalidHold);
        }
        let grenadiers_core::cpu::survival::SurvivalPlan {
            control,
            lease,
            execution,
        } = *plan;
        let topology_contact = matches!(
            execution.as_ref(),
            Some(ExecutionCertificate::ContactPlant(_))
        );
        if topology_contact && !self.supports_contact_topology() {
            return self.reject(AdmissionRejection::InvalidShape);
        }
        if execution.as_ref().is_some_and(|certificate| {
            certificate.contact_plant_requires_blocked_birth_support() == Some(true)
        }) && !self.supports_blocked_birth_contact()
        {
            return self.reject(AdmissionRejection::InvalidShape);
        }
        let contact_required = (result.basis.parts().rules.version == 8
            || (matches!(result.basis.parts().rules.version, 9 | 10)
                && result.basis.respawn_lairs().is_some()))
            && (control.intent.manual.is_some()
                || control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop);
        let contact = match contact {
            Some(contact)
                if contact.safe_until() > current_tick
                    && contact.observation_fits(latest, &mut WorkBudget::new(65_536)) =>
            {
                Some(contact)
            }
            Some(_) => return self.reject(AdmissionRejection::Contact),
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
            return self.reject(AdmissionRejection::StrategicPlant);
        }
        if (escape_origin
            && (control.intent.manual.is_some()
                || execution.is_some()
                || (control.intent.movement != grenadiers_core::cpu::MovementIntent::Stop
                    && lease.is_none())))
            || (execution.is_some() && (lease.is_some() || invalid_manual))
        {
            return self.reject(AdmissionRejection::InvalidShape);
        }
        let committed_pulse = match lease {
            Some(lease) => {
                match CommittedPulse::admit(lease, result.basis.clone(), latest, current_tick) {
                    Ok(pulse) => Some(pulse),
                    Err(_) => return self.reject(AdmissionRejection::Pulse),
                }
            }
            None => None,
        };
        let committed_route = match execution {
            Some(execution) => match CommittedRoute::admit_execution(
                execution,
                result.basis,
                latest,
                current_tick,
                control.intent.movement,
            ) {
                Ok(route) => Some(route),
                Err(_) => return self.reject(AdmissionRejection::Route),
            },
            None => None,
        };
        if contact_required
            && contact.is_none()
            && committed_route
                .as_ref()
                .is_none_or(|route| !route.has_contact_authority())
        {
            return self.reject(AdmissionRejection::Contact);
        }
        if !self.admit_continuation_decision(pending.decision, command_epoch) {
            return self.reject(AdmissionRejection::DuplicateDecision);
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
            refuge_handoff: refuge_handoff.is_some(),
        })
    }

    fn reject(&mut self, reason: AdmissionRejection) -> Option<AcceptedPlan> {
        #[cfg(test)]
        match reason {
            AdmissionRejection::ExpiredOrWrongDecision => {
                self.admission_trace.expired_or_wrong_decision += 1;
            }
            AdmissionRejection::MissedManualTick => self.admission_trace.missed_manual_tick += 1,
            AdmissionRejection::StaleScene => self.admission_trace.stale_scene += 1,
            AdmissionRejection::InvalidHold => self.admission_trace.invalid_hold += 1,
            AdmissionRejection::Contact => self.admission_trace.contact += 1,
            AdmissionRejection::StrategicPlant => self.admission_trace.strategic_plant += 1,
            AdmissionRejection::InvalidShape => self.admission_trace.invalid_shape += 1,
            AdmissionRejection::Pulse => self.admission_trace.pulse += 1,
            AdmissionRejection::Route => self.admission_trace.route += 1,
            AdmissionRejection::DuplicateDecision => self.admission_trace.duplicate_decision += 1,
            AdmissionRejection::RefugeHandoff => self.admission_trace.refuge_handoff += 1,
        }
        #[cfg(not(test))]
        let _ = reason;
        None
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn admission_trace_for_test(&self) -> AdmissionTrace {
        self.admission_trace.clone()
    }
}
