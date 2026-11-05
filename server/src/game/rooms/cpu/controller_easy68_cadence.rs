use grenadiers_core::cpu::survival::PulseStatus;
use grenadiers_core::cpu::{Intent, Observation, ObservedActor, WorkBudget, WorldId};
use grenadiers_core::game::{Position, Tick};
use grenadiers_core::room::CpuProfile;

use super::{AcceptedPlan, CpuController};

const PAUSE_TICKS: u64 = 12;
const GUARD_ALLOWANCE: u32 = 65_536;

#[derive(Clone, Copy)]
struct CandidateKey {
    world: WorldId,
    command_epoch: u64,
}

#[derive(Clone, Copy)]
enum Phase {
    Dormant(CandidateKey),
    AwaitingStop {
        key: CandidateKey,
        validated_at: Option<Tick>,
    },
    Pausing {
        key: CandidateKey,
        deadline: Tick,
        pose: Position,
        validated_at: Tick,
    },
}

#[derive(Default)]
pub(super) struct Easy68CadenceState {
    phase: Option<Phase>,
    last_guard_at: Option<Tick>,
    #[cfg(test)]
    metrics: Easy68CadenceMetrics,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::game::rooms) struct Easy68CadenceMetrics {
    pub pause_armed: u64,
    pub pause_completed: u64,
    pub planning_deferred: u64,
    pub planning_bypassed: u64,
    pub guard_work: u64,
}

impl CpuController {
    pub(super) fn arm_easy68_cadence_after(&mut self, accepted: &AcceptedPlan, actual: Intent) {
        if self.profile != CpuProfile::CERTIFIED_CADENCE_V68_EASY {
            return;
        }
        let eligible = accepted.committed_pulse.is_some()
            && accepted.committed_route.is_none()
            && accepted.certified_manual.is_none()
            && !accepted.escape_origin
            && !accepted.refuge_handoff
            && accepted.intent == actual
            && actual.manual.is_none()
            && actual.movement != grenadiers_core::cpu::MovementIntent::Stop
            && self.escape_origin.is_none()
            && self.prepared_continuation.is_none()
            && self.continuation.is_none()
            && self.committed_teleporter.is_none();
        if !eligible {
            self.bypass_easy68_cadence();
            return;
        }
        self.easy68_cadence.phase = Some(Phase::Dormant(CandidateKey {
            world: accepted.decision.world,
            command_epoch: accepted.decision.command_epoch,
        }));
        self.easy68_cadence.last_guard_at = None;
        #[cfg(test)]
        {
            self.easy68_cadence.metrics.pause_armed += 1;
        }
    }

    pub(super) fn observe_easy68_cadence(&mut self, observation: &Observation) {
        if self.profile != CpuProfile::CERTIFIED_CADENCE_V68_EASY {
            self.easy68_cadence = Easy68CadenceState::default();
            return;
        }
        let Some(phase) = self.easy68_cadence.phase else {
            return;
        };
        let key = phase.key();
        if key.world != observation.parts().world || self.easy68_context_conflicts(phase) {
            self.bypass_easy68_cadence();
            return;
        }
        if let Phase::Pausing {
            key,
            deadline,
            pose,
            ..
        } = phase
            && observation.parts().tick >= deadline
        {
            if recipient_pose(observation) != Some(pose) {
                self.bypass_easy68_cadence();
                return;
            }
            self.easy68_cadence.phase = Some(Phase::Pausing {
                key,
                deadline,
                pose,
                validated_at: observation.parts().tick,
            });
            return;
        }
        let Some(through) = phase.guard_through(observation.parts().tick) else {
            self.bypass_easy68_cadence();
            return;
        };
        if !self.easy68_guard(observation, through) {
            self.bypass_easy68_cadence();
            return;
        }
        self.easy68_cadence.last_guard_at = Some(observation.parts().tick);
        match phase {
            Phase::Dormant(_) => {}
            Phase::AwaitingStop { key, .. } => {
                let Some(stopped) = self.settled_stop_tick else {
                    self.easy68_cadence.phase = Some(Phase::AwaitingStop {
                        key,
                        validated_at: self.stop_admission.map(|_| observation.parts().tick),
                    });
                    return;
                };
                let Some(deadline) = stopped.0.checked_add(PAUSE_TICKS).map(Tick) else {
                    self.bypass_easy68_cadence();
                    return;
                };
                let Some(pose) = recipient_pose(observation) else {
                    self.bypass_easy68_cadence();
                    return;
                };
                self.easy68_cadence.phase = Some(Phase::Pausing {
                    key,
                    deadline,
                    pose,
                    validated_at: observation.parts().tick,
                });
            }
            Phase::Pausing {
                key,
                deadline,
                pose,
                ..
            } => {
                if recipient_pose(observation) != Some(pose) {
                    self.bypass_easy68_cadence();
                    return;
                }
                self.easy68_cadence.phase = Some(Phase::Pausing {
                    key,
                    deadline,
                    pose,
                    validated_at: observation.parts().tick,
                });
            }
        }
    }

    pub(super) fn easy68_pulse_status(&mut self, status: PulseStatus) {
        if self.profile != CpuProfile::CERTIFIED_CADENCE_V68_EASY {
            return;
        }
        match (self.easy68_cadence.phase, status) {
            (Some(Phase::Dormant(key)), PulseStatus::Completed) => {
                let pausing = self
                    .settled_stop_tick
                    .and_then(|stopped| stopped.0.checked_add(PAUSE_TICKS).map(Tick))
                    .zip(self.latest_observation.as_ref())
                    .and_then(|(deadline, observation)| {
                        let validated_at = observation.parts().tick;
                        (self.easy68_cadence.last_guard_at == Some(validated_at))
                            .then_some(())
                            .and_then(|()| recipient_pose(observation))
                            .map(|pose| (deadline, pose, validated_at))
                    });
                self.easy68_cadence.phase = if let Some((deadline, pose, validated_at)) = pausing {
                    Some(Phase::Pausing {
                        key,
                        deadline,
                        pose,
                        validated_at,
                    })
                } else {
                    Some(Phase::AwaitingStop {
                        key,
                        validated_at: self.stop_admission.and(self.easy68_cadence.last_guard_at),
                    })
                };
            }
            (Some(_), PulseStatus::Invalidated) => self.bypass_easy68_cadence(),
            _ => {}
        }
    }

    pub(super) fn validate_easy68_epoch(&mut self, command_epoch: u64) {
        if self
            .easy68_cadence
            .phase
            .is_some_and(|phase| phase.key().command_epoch != command_epoch)
        {
            self.bypass_easy68_cadence();
        }
    }

    pub(super) fn defer_easy68_planning(
        &mut self,
        observation: &Observation,
        command_epoch: u64,
    ) -> bool {
        if let Some(Phase::AwaitingStop {
            key,
            validated_at: Some(validated_at),
        }) = self.easy68_cadence.phase
        {
            if key.world != observation.parts().world
                || key.command_epoch != command_epoch
                || validated_at != observation.parts().tick
            {
                self.bypass_easy68_cadence();
                return false;
            }
            #[cfg(test)]
            {
                self.easy68_cadence.metrics.planning_deferred += 1;
            }
            return true;
        }
        let Some(Phase::Pausing {
            key,
            deadline,
            pose,
            validated_at,
        }) = self.easy68_cadence.phase
        else {
            return false;
        };
        if key.world != observation.parts().world
            || key.command_epoch != command_epoch
            || validated_at != observation.parts().tick
            || recipient_pose(observation) != Some(pose)
        {
            self.bypass_easy68_cadence();
            return false;
        }
        if observation.parts().tick >= deadline {
            self.easy68_cadence.phase = None;
            #[cfg(test)]
            {
                self.easy68_cadence.metrics.pause_completed += 1;
            }
            return false;
        }
        #[cfg(test)]
        {
            self.easy68_cadence.metrics.planning_deferred += 1;
        }
        true
    }

    fn easy68_context_conflicts(&self, phase: Phase) -> bool {
        self.escaping
            || self.escape_origin.is_some()
            || self.committed_route.is_some()
            || self.prepared_continuation.is_some()
            || self.continuation.is_some()
            || self.committed_teleporter.is_some()
            || (!matches!(phase, Phase::Dormant(_)) && self.committed_pulse.is_some())
    }

    fn easy68_guard(&mut self, observation: &Observation, through: Tick) -> bool {
        let mut work = WorkBudget::new(GUARD_ALLOWANCE);
        let allowed = grenadiers_core::cpu::can_pause_easy_v68(observation, through, &mut work);
        #[cfg(test)]
        {
            self.easy68_cadence.metrics.guard_work += u64::from(work.spent());
        }
        allowed
    }

    fn bypass_easy68_cadence(&mut self) {
        if self.easy68_cadence.phase.take().is_some() {
            self.easy68_cadence.last_guard_at = None;
            #[cfg(test)]
            {
                self.easy68_cadence.metrics.planning_bypassed += 1;
            }
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) const fn easy68_cadence_metrics_for_test(
        &self,
    ) -> Easy68CadenceMetrics {
        self.easy68_cadence.metrics
    }
}

impl Phase {
    const fn key(self) -> CandidateKey {
        match self {
            Self::Dormant(key) | Self::AwaitingStop { key, .. } | Self::Pausing { key, .. } => key,
        }
    }

    fn guard_through(self, current: Tick) -> Option<Tick> {
        match self {
            Self::Pausing { deadline, .. } => Some(deadline),
            Self::Dormant(_) | Self::AwaitingStop { .. } => {
                current.0.checked_add(PAUSE_TICKS).map(Tick)
            }
        }
    }
}

fn recipient_pose(observation: &Observation) -> Option<Position> {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor)
                if actor.seat == observation.parts().recipient && actor.alive =>
            {
                Some(actor.position)
            }
            _ => None,
        })
}

#[cfg(test)]
#[path = "controller_easy68_cadence_tests.rs"]
mod tests;
