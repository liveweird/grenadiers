use std::sync::mpsc::TryRecvError;

use grenadiers_core::cpu::survival::{
    MAX_PLAN_AGE_TICKS, PreparedTeleporterStep, TELEPORTER_ADMISSION_ALLOWANCE,
    TeleporterStepCommand, TeleporterStepStatus,
};
use grenadiers_core::cpu::{CpuRng, DecisionId, MovementIntent, Observation, WorkBudget};
use grenadiers_core::game::Tick;

use super::super::worker::WorkerPlan;
use super::CpuController;

pub(in crate::game::rooms) enum TeleporterService {
    Idle,
    Controlled,
    Submit(TeleporterSubmission),
    Invalidated,
    Completed,
}

pub(in crate::game::rooms) struct TeleporterSubmission {
    command: TeleporterStepCommand,
    kind: SubmissionKind,
}

enum SubmissionKind {
    Admission(Box<TeleporterAdmission>),
    Stop,
}

struct TeleporterAdmission {
    prepared: PreparedTeleporterStep,
    rng: CpuRng,
    decision: DecisionId,
}

impl TeleporterSubmission {
    pub(in crate::game::rooms) const fn movement(&self) -> MovementIntent {
        self.command.movement()
    }

    pub(in crate::game::rooms) const fn admission_tick(&self) -> Tick {
        self.command.admission_tick()
    }

    pub(in crate::game::rooms) const fn is_stop(&self) -> bool {
        matches!(self.kind, SubmissionKind::Stop)
    }
}

impl CpuController {
    pub(in crate::game::rooms) const fn teleporter_controlled(&self) -> bool {
        self.committed_teleporter.is_some()
    }

    pub(in crate::game::rooms) fn service_teleporter(
        &mut self,
        current_tick: Tick,
        command_epoch: u64,
    ) -> TeleporterService {
        if let Some(committed) = self.committed_teleporter.as_mut() {
            let Some(key) = self.memory.as_ref().map(|memory| memory.key()) else {
                self.committed_teleporter = None;
                return TeleporterService::Invalidated;
            };
            return match committed.poll_command(current_tick, key, command_epoch) {
                TeleporterStepStatus::Moving | TeleporterStepStatus::WaitForStoppedObservation => {
                    TeleporterService::Controlled
                }
                TeleporterStepStatus::Submit(command) => {
                    TeleporterService::Submit(TeleporterSubmission {
                        command,
                        kind: SubmissionKind::Stop,
                    })
                }
                TeleporterStepStatus::Completed(_) => {
                    self.finish_teleporter_completion();
                    TeleporterService::Completed
                }
                TeleporterStepStatus::Invalidated => {
                    self.committed_teleporter = None;
                    TeleporterService::Invalidated
                }
            };
        }

        let Some(pending) = self.pending.as_mut() else {
            return TeleporterService::Idle;
        };
        if pending.completed.is_none() {
            match pending.receiver.try_recv() {
                Ok(result) => pending.completed = Some(result),
                Err(TryRecvError::Empty) => return TeleporterService::Idle,
                Err(TryRecvError::Disconnected) => {
                    self.pending = None;
                    return TeleporterService::Idle;
                }
            }
        }
        let Some(result) = pending.completed.as_ref() else {
            return TeleporterService::Idle;
        };
        if !matches!(result.outcome, WorkerPlan::TeleporterStep(_)) {
            return TeleporterService::Idle;
        }
        let decision = pending.decision;
        let invalid_identity = !self.supports_teleporter_step()
            || decision.command_epoch != command_epoch
            || self.world != Some(decision.world)
            || current_tick
                .0
                .checked_sub(decision.observation_tick.0)
                .is_none_or(|age| age > MAX_PLAN_AGE_TICKS);
        if invalid_identity {
            self.cancel();
            return TeleporterService::Invalidated;
        }
        // Worker completion between publications is retained. Admission is
        // attempted only against the actual fresh publication for this tick.
        let Some(latest) = self.latest_observation.as_ref() else {
            return TeleporterService::Controlled;
        };
        if latest.parts().tick != current_tick || latest.parts().tick <= decision.observation_tick {
            return TeleporterService::Controlled;
        }

        let latest = latest.clone();
        let Some(pending) = self.pending.take() else {
            return TeleporterService::Invalidated;
        };
        let Some(result) = pending.completed else {
            return TeleporterService::Invalidated;
        };
        let WorkerPlan::TeleporterStep(plan) = result.outcome else {
            return TeleporterService::Invalidated;
        };
        let Some(settled_stop_tick) = self.settled_stop_tick else {
            return TeleporterService::Invalidated;
        };
        let mut work = WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE);
        let admit = if self.supports_stable_dead_roster() {
            PreparedTeleporterStep::admit_v48
        } else {
            PreparedTeleporterStep::admit
        };
        let Ok(mut prepared) = admit(
            *plan,
            result.basis,
            &latest,
            settled_stop_tick,
            decision,
            current_tick,
            &mut work,
        ) else {
            return TeleporterService::Invalidated;
        };
        let Ok(command) = prepared.command() else {
            return TeleporterService::Invalidated;
        };
        TeleporterService::Submit(TeleporterSubmission {
            command,
            kind: SubmissionKind::Admission(Box::new(TeleporterAdmission {
                prepared,
                rng: result.rng,
                decision,
            })),
        })
    }

    pub(in crate::game::rooms) fn commit_teleporter(
        &mut self,
        submission: TeleporterSubmission,
    ) -> bool {
        match submission.kind {
            SubmissionKind::Admission(admission) => {
                let TeleporterAdmission {
                    prepared,
                    rng,
                    decision,
                } = *admission;
                if !self.admit_continuation_decision(decision, submission.command.command_epoch()) {
                    return false;
                }
                let Ok(committed) = prepared.commit(submission.command) else {
                    return false;
                };
                self.rng = Some(rng);
                self.clear_escape_origin();
                self.committed_pulse = None;
                self.committed_route = None;
                self.committed_contact = None;
                self.committed_route_epoch = None;
                self.committed_teleporter = Some(committed);
                true
            }
            SubmissionKind::Stop => self
                .committed_teleporter
                .as_mut()
                .is_some_and(|committed| committed.record_stop(submission.command)),
        }
    }

    pub(in crate::game::rooms) fn observe_teleporter(
        &mut self,
        observation: &Observation,
    ) -> Option<TeleporterStepStatus> {
        let settled = self.settled_stop_tick?;
        let committed = self.committed_teleporter.as_mut()?;
        let mut work = WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE);
        let status = committed.observe_stopped(observation, settled, &mut work);
        match status {
            TeleporterStepStatus::Completed(_) => self.finish_teleporter_completion(),
            // Retain the revoked authority until owner service observes it and
            // performs the mandatory physical input fence.
            TeleporterStepStatus::Invalidated => {}
            _ => {}
        }
        Some(status)
    }

    pub(in crate::game::rooms) fn invalidate_teleporter(&mut self) {
        self.committed_teleporter = None;
    }

    fn finish_teleporter_completion(&mut self) {
        if let Some(memory) = self
            .committed_teleporter
            .as_ref()
            .and_then(|committed| committed.next_memory())
        {
            self.memory = Some(memory);
        }
        self.committed_teleporter = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rooms::cpu::{CpuPlanner, worker_teleporter_tests::teleporter_observation};
    use grenadiers_core::{game::SeatId, room::CpuProfile};

    #[test]
    fn ready_step_waits_for_a_fresh_publication_before_admission() {
        let planner = CpuPlanner::without_workers();
        let mut controller = CpuController::new(
            planner.clone(),
            SeatId::Two,
            7,
            [46; 32],
            CpuProfile::TELEPORTER_V46_STANDARD,
        );
        let basis = at_tick(teleporter_observation(), Tick(21));
        let initial = at_tick(basis.clone(), Tick(10));
        assert!(controller.accept_observation(&initial));
        controller.on_stop_admitted(Tick(10));
        assert!(controller.accept_observation(&at_tick(basis.clone(), Tick(11))));
        assert!(controller.accept_observation(&basis));
        assert!(controller.submit(basis.clone(), 3));
        assert_eq!(planner.pump_for_test(1).delivered, 1);

        assert!(matches!(
            controller.service_teleporter(Tick(22), 3),
            TeleporterService::Controlled
        ));
        assert!(controller.has_pending());

        let fresh = at_tick(basis, Tick(22));
        assert!(controller.accept_observation(&fresh));
        let submission = match controller.service_teleporter(Tick(22), 3) {
            TeleporterService::Submit(value) => value,
            TeleporterService::Idle => panic!("unexpected idle Step service"),
            TeleporterService::Controlled => panic!("unexpected controlled Step service"),
            TeleporterService::Invalidated => panic!("fresh Step admission invalidated"),
            TeleporterService::Completed => panic!("unexpected completed Step service"),
        };
        assert_eq!(submission.admission_tick(), Tick(22));
        assert!(!submission.is_stop());
        assert!(controller.commit_teleporter(submission));
        controller.on_movement_admitted();
        assert!(controller.teleporter_controlled());

        assert!(matches!(
            controller.service_teleporter(Tick(22), 3),
            TeleporterService::Controlled
        ));
        let stop = match controller.service_teleporter(Tick(23), 3) {
            TeleporterService::Submit(value) => value,
            _ => panic!("the next physical update must submit the sealed Stop"),
        };
        assert_eq!(stop.admission_tick(), Tick(23));
        assert!(stop.is_stop());
        assert!(controller.commit_teleporter(stop));
        controller.on_stop_admitted(Tick(23));
        assert!(controller.teleporter_controlled());
    }

    fn at_tick(observation: Observation, tick: Tick) -> Observation {
        observation
            .map_parts(|parts| {
                if tick >= parts.tick {
                    let elapsed = (tick.0 - parts.tick.0) as u16;
                    parts.rules.running_updates =
                        parts.rules.running_updates.saturating_add(elapsed);
                    parts.rules.remaining_ticks =
                        parts.rules.remaining_ticks.saturating_sub(elapsed);
                } else {
                    let elapsed = (parts.tick.0 - tick.0) as u16;
                    parts.rules.running_updates =
                        parts.rules.running_updates.saturating_sub(elapsed);
                    parts.rules.remaining_ticks =
                        parts.rules.remaining_ticks.saturating_add(elapsed);
                }
                parts.tick = tick;
            })
            .unwrap()
    }
}
