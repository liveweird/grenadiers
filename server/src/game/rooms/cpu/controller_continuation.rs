use std::sync::mpsc::TryRecvError;

use grenadiers_core::cpu::routing::CONTINUATION_RENEWAL_ALLOWANCE;
use grenadiers_core::cpu::survival::{
    ContinuationCommand, ContinuationPublication, ContinuationStatus, PendingContinuation,
    PreparedRenewal, continuation_recovery_memory,
};
use grenadiers_core::cpu::{MovementIntent, Observation, WorkBudget};
use grenadiers_core::game::Tick;

use super::super::worker::WorkerPlan;
use super::continuation_types::{ContinuationAuthority, PreparedAdmission};
use super::{CpuController, Pending};

pub(in crate::game::rooms) enum ContinuationService {
    Idle,
    Controlled,
    Submit(ContinuationSubmission),
    Invalidated,
    Completed,
}

pub(in crate::game::rooms) struct ContinuationSubmission {
    command: ContinuationCommand,
    kind: SubmissionKind,
}

enum SubmissionKind {
    Admission(PreparedAdmission),
    Renewal(PreparedRenewal),
    Command,
}

impl ContinuationSubmission {
    pub(in crate::game::rooms) const fn movement(&self) -> MovementIntent {
        self.command.movement()
    }

    pub(in crate::game::rooms) const fn admission_tick(&self) -> Tick {
        self.command.admission_tick()
    }
}

impl CpuController {
    #[cfg(test)]
    pub(in crate::game::rooms) const fn has_continuation_authority_for_test(&self) -> bool {
        self.continuation.is_some()
    }

    #[cfg(test)]
    pub(in crate::game::rooms) const fn continuation_trace_for_test(
        &self,
    ) -> super::continuation_types::ContinuationTrace {
        self.continuation_trace
    }

    pub(in crate::game::rooms) fn observe_continuation_publication(
        &mut self,
        sequence: u64,
        observation: Option<&Observation>,
    ) -> bool {
        self.publication_sequence = Some(sequence);
        let pending_valid = self
            .pending
            .as_mut()
            .and_then(|pending| pending.continuation.as_mut())
            .is_none_or(|clock| clock.observe(sequence, observation));
        if !pending_valid {
            self.cancel_continuation_attempt();
            return false;
        }
        if self.prepared_continuation.is_some() {
            self.invalidate_continuation();
            return false;
        }
        let Some(authority) = self.continuation.as_mut() else {
            return true;
        };
        let Some(current) = observation else {
            self.invalidate_continuation();
            return false;
        };
        let Ok(publication) = ContinuationPublication::new(
            authority.decision,
            authority.key,
            sequence,
            current.parts().tick,
        ) else {
            self.invalidate_continuation();
            return false;
        };
        let mut work = WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE);
        match authority
            .committed
            .prepare_grant(current, publication, &mut work)
        {
            Ok(prepared) => {
                authority.renewal = Some(prepared);
                true
            }
            Err(_) => {
                self.invalidate_continuation();
                false
            }
        }
    }

    pub(in crate::game::rooms) fn service_continuation(
        &mut self,
        current_tick: Tick,
        command_epoch: u64,
    ) -> ContinuationService {
        if std::mem::take(&mut self.continuation_fence_required) {
            return ContinuationService::Invalidated;
        }
        if self.prepared_continuation.is_none() && self.continuation.is_none() {
            self.prepare_worker_continuation(current_tick, command_epoch);
        }
        if let Some(mut prepared) = self.prepared_continuation.take() {
            let Ok(command) = prepared.prepared.command() else {
                self.invalidate_continuation();
                return ContinuationService::Invalidated;
            };
            return ContinuationService::Submit(ContinuationSubmission {
                command,
                kind: SubmissionKind::Admission(prepared),
            });
        }
        let Some(authority) = self.continuation.as_mut() else {
            return if self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.continuation.is_some())
            {
                ContinuationService::Controlled
            } else {
                ContinuationService::Idle
            };
        };
        if let Some(mut renewal) = authority.renewal.take() {
            let Ok(command) = renewal.command() else {
                self.invalidate_continuation();
                return ContinuationService::Invalidated;
            };
            return ContinuationService::Submit(ContinuationSubmission {
                command,
                kind: SubmissionKind::Renewal(renewal),
            });
        }
        match authority
            .committed
            .poll_command(current_tick, command_epoch)
        {
            ContinuationStatus::Submit(command) => {
                ContinuationService::Submit(ContinuationSubmission {
                    command,
                    kind: SubmissionKind::Command,
                })
            }
            ContinuationStatus::Continue | ContinuationStatus::Wait => {
                ContinuationService::Controlled
            }
            ContinuationStatus::RenewalRequired => {
                self.invalidate_continuation();
                self.continuation_fence_required = false;
                ContinuationService::Invalidated
            }
            ContinuationStatus::Completed => {
                self.continuation = None;
                ContinuationService::Completed
            }
            ContinuationStatus::Invalidated => {
                self.invalidate_continuation();
                self.continuation_fence_required = false;
                ContinuationService::Invalidated
            }
        }
    }

    pub(in crate::game::rooms) fn commit_continuation(
        &mut self,
        submission: ContinuationSubmission,
    ) -> bool {
        #[cfg(test)]
        let movement = submission.command.movement();
        #[cfg(test)]
        let command_tick = submission.command.admission_tick();
        match submission.kind {
            SubmissionKind::Admission(prepared) => {
                #[cfg(test)]
                let admission_tick = submission.command.admission_tick();
                let Ok(committed) = prepared.prepared.commit(submission.command) else {
                    self.invalidate_continuation();
                    return false;
                };
                let Some(memory) = self.memory.as_ref() else {
                    return false;
                };
                self.memory = Some(continuation_recovery_memory(memory));
                self.clear_escape_origin();
                self.committed_pulse = None;
                self.committed_route = None;
                self.committed_contact = None;
                self.committed_route_epoch = None;
                self.continuation = Some(ContinuationAuthority {
                    committed,
                    decision: prepared.decision,
                    key: prepared.key,
                    renewal: None,
                });
                #[cfg(test)]
                {
                    self.continuation_trace.admission_tick = Some(admission_tick);
                    self.trace_first_movement(movement, command_tick);
                }
                true
            }
            SubmissionKind::Renewal(prepared) => {
                let committed = self.continuation.as_mut().is_some_and(|authority| {
                    authority
                        .committed
                        .commit_grant(prepared, submission.command)
                });
                #[cfg(test)]
                if committed {
                    self.continuation_trace.renewal_commits += 1;
                    self.trace_first_movement(movement, command_tick);
                }
                committed
            }
            SubmissionKind::Command => {
                let committed = self.continuation.as_mut().is_some_and(|authority| {
                    authority.committed.commit_command(submission.command)
                });
                #[cfg(test)]
                if committed {
                    self.continuation_trace.command_commits += 1;
                    self.trace_first_movement(movement, command_tick);
                }
                committed
            }
        }
    }

    #[cfg(test)]
    fn trace_first_movement(&mut self, movement: MovementIntent, tick: Tick) {
        if matches!(movement, MovementIntent::Direction(_))
            && self
                .continuation_trace
                .first_movement_command_tick
                .is_none()
        {
            self.continuation_trace.first_movement_command_tick = Some(tick);
        }
    }

    pub(in crate::game::rooms) fn invalidate_continuation(&mut self) {
        self.continuation_fence_required |=
            self.continuation.is_some() || self.prepared_continuation.is_some();
        if let Some(authority) = self.continuation.as_mut() {
            authority.committed.revoke();
        }
        self.continuation = None;
        self.prepared_continuation = None;
    }

    pub(in crate::game::rooms) fn finish_continuation_fence(&mut self) {
        self.invalidate_continuation();
        self.continuation_fence_required = false;
    }

    fn prepare_worker_continuation(&mut self, current_tick: Tick, command_epoch: u64) {
        let Some(pending) = self.pending.as_mut() else {
            return;
        };
        if pending.completed.is_none() {
            match pending.receiver.try_recv() {
                Ok(result) => pending.completed = Some(result),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.cancel_continuation_attempt();
                    return;
                }
            }
        }
        if pending
            .completed
            .as_ref()
            .is_some_and(|result| matches!(result.outcome, WorkerPlan::Ordinary { .. }))
        {
            pending.continuation = None;
            return;
        }
        if pending.completed.is_none() {
            return;
        }
        let admission_tick = pending.decision.observation_tick.0.checked_add(6).map(Tick);
        if current_tick < admission_tick.unwrap_or(Tick(0)) {
            return;
        }
        let Some(pending) = self.pending.take() else {
            return;
        };
        if admission_tick != Some(current_tick)
            || pending.decision.command_epoch != command_epoch
            || pending
                .continuation
                .as_ref()
                .is_none_or(|clock| clock.revoked || clock.observed.len() != 2)
        {
            self.recover_cancelled_continuation();
            return;
        }
        self.prepare_admission(pending, command_epoch);
    }

    fn prepare_admission(&mut self, pending: Pending, command_epoch: u64) {
        let Some(result) = pending.completed else {
            return;
        };
        let WorkerPlan::Continuation(certificate) = result.outcome else {
            return;
        };
        if result
            .escape_origin
            .as_ref()
            .zip(self.escape_origin.as_ref())
            .is_none_or(|(candidate, current)| !candidate.same_origin(current))
        {
            self.recover_cancelled_continuation();
            return;
        }
        let Some(clock) = pending.continuation else {
            self.recover_cancelled_continuation();
            return;
        };
        let Some(key) = self.memory.as_ref().map(|memory| memory.key()) else {
            return;
        };
        let Ok(mut core_pending) =
            PendingContinuation::new(certificate, pending.decision, key, clock.basis_sequence)
        else {
            self.recover_cancelled_continuation();
            return;
        };
        let mut prepared = None;
        for (sequence, observation) in clock.observed {
            let mut work = WorkBudget::new(CONTINUATION_RENEWAL_ALLOWANCE);
            let Ok(publication) = ContinuationPublication::new(
                pending.decision,
                key,
                sequence,
                observation.parts().tick,
            ) else {
                self.recover_cancelled_continuation();
                return;
            };
            match core_pending.observe(&observation, publication, &mut work) {
                Ok(value) => prepared = value.or(prepared),
                Err(_) => {
                    self.recover_cancelled_continuation();
                    return;
                }
            }
        }
        let Some(prepared) = prepared else {
            self.recover_cancelled_continuation();
            return;
        };
        if !self.admit_continuation_decision(pending.decision, command_epoch) {
            self.recover_cancelled_continuation();
            return;
        }
        self.prepared_continuation = Some(PreparedAdmission {
            prepared,
            decision: pending.decision,
            key,
        });
    }

    fn cancel_continuation_attempt(&mut self) {
        self.cancel();
    }

    fn recover_cancelled_continuation(&mut self) {
        self.prepared_continuation = None;
    }
}
