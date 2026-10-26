use std::fmt;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, TryRecvError, sync_channel},
};

use grenadiers_core::cpu::forecast::ObservedHazardMemory;
use grenadiers_core::cpu::policy::{ControlPlan, EscapeExecution, MAX_HOLD_TICKS, PolicyError};
use grenadiers_core::cpu::survival::{CommittedPulse, CommittedRoute, MAX_PLAN_AGE_TICKS};
use grenadiers_core::cpu::{
    AdmissionContext, CpuMemory, CpuRng, DecisionId, Intent, Observation, WorldId,
    policy::UtilityMemory,
};
use grenadiers_core::game::Tick;

use super::worker::{CpuPlanner, PlanJob, PlanResult};

struct Pending {
    decision: DecisionId,
    receiver: Receiver<PlanResult>,
    live: Arc<AtomicBool>,
}

pub(in crate::game::rooms) struct AcceptedPlan {
    pub(in crate::game::rooms) decision: DecisionId,
    pub(in crate::game::rooms) intent: Intent,
    pub(in crate::game::rooms) hold_ticks: u8,
    next_memory: UtilityMemory,
    next_rng: CpuRng,
    committed_pulse: Option<CommittedPulse>,
    committed_route: Option<CommittedRoute>,
}

#[derive(Clone, Copy)]
struct StopAdmission {
    tick: Tick,
}

pub(in crate::game::rooms) struct CpuController {
    planner: CpuPlanner,
    room_instance: u64,
    private_seed: [u8; 32],
    world: Option<WorldId>,
    admission_memory: Option<CpuMemory>,
    memory: Option<UtilityMemory>,
    hazard_memory: Option<ObservedHazardMemory>,
    rng: Option<CpuRng>,
    pending: Option<Pending>,
    request_serial: u64,
    escaping: bool,
    last_observation_tick: Option<Tick>,
    latest_observation: Option<Observation>,
    stop_admission: Option<StopAdmission>,
    settled_stop_tick: Option<Tick>,
    committed_pulse: Option<CommittedPulse>,
    committed_route: Option<CommittedRoute>,
    committed_route_epoch: Option<u64>,
}

impl fmt::Debug for CpuController {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CpuController")
            .field("room_instance", &self.room_instance)
            .field("private_seed", &"<redacted>")
            .field("world", &self.world)
            .field("request_serial", &self.request_serial)
            .field("pending", &self.pending.is_some())
            .field("escaping", &self.escaping)
            .finish()
    }
}

impl CpuController {
    pub(in crate::game::rooms) fn new(
        planner: CpuPlanner,
        room_instance: u64,
        private_seed: [u8; 32],
    ) -> Self {
        Self {
            planner,
            room_instance,
            private_seed,
            world: None,
            admission_memory: None,
            memory: None,
            hazard_memory: None,
            rng: None,
            pending: None,
            request_serial: 0,
            escaping: false,
            last_observation_tick: None,
            latest_observation: None,
            stop_admission: None,
            settled_stop_tick: None,
            committed_pulse: None,
            committed_route: None,
            committed_route_epoch: None,
        }
    }

    pub(in crate::game::rooms) fn submit(
        &mut self,
        observation: Observation,
        command_epoch: u64,
    ) -> bool {
        if self.escaping
            || self.committed_pulse.is_some()
            || self.committed_route.is_some()
            || self.pending.is_some()
            || self.world != Some(observation.parts().world)
        {
            return false;
        }
        let Some(settled_stop_tick) = self.settled_stop_tick else {
            return false;
        };
        let Some(serial) = self.request_serial.checked_add(1) else {
            return false;
        };
        let decision = DecisionId {
            world: observation.parts().world,
            command_epoch,
            observation_tick: observation.parts().tick,
            checked_request_serial: serial,
        };
        let (sender, receiver) = sync_channel(1);
        let live = Arc::new(AtomicBool::new(true));
        let job = PlanJob {
            observation,
            memory: *self.memory.as_ref().expect("world memory initialized"),
            hazard_memory: self
                .hazard_memory
                .as_ref()
                .expect("world hazard memory initialized")
                .clone(),
            settled_stop_tick,
            rng: self.rng.as_ref().expect("world rng initialized").clone(),
            live: Arc::clone(&live),
            result: sender,
        };
        if !self.planner.try_submit(job) {
            return false;
        }
        self.request_serial = serial;
        self.pending = Some(Pending {
            decision,
            receiver,
            live,
        });
        true
    }

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
        let result = match pending.receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return None,
            Err(TryRecvError::Disconnected) => {
                self.pending = None;
                return None;
            }
        };
        let pending = self.pending.take().expect("pending was checked");
        let latest = self.latest_observation.as_ref()?;
        if self.settled_stop_tick.is_none()
            || (result.plan.route.is_none()
                && !grenadiers_core::cpu::survival::can_admit_ordinary(&result.basis, latest))
        {
            return None;
        }
        let current = AdmissionContext {
            world: self.world?,
            command_epoch,
            observation_tick: pending.decision.observation_tick,
            checked_request_serial: self.request_serial,
        };
        if !(1..=MAX_HOLD_TICKS).contains(&result.plan.control.hold_ticks)
            || grenadiers_core::cpu::admit_decision(
                self.admission_memory.as_mut()?,
                current,
                pending.decision,
                &mut grenadiers_core::cpu::WorkBudget::new(1),
            ) != grenadiers_core::cpu::DecisionAdmission::Accepted
        {
            return None;
        }
        let grenadiers_core::cpu::survival::SurvivalPlan {
            control,
            lease,
            route,
        } = result.plan;
        if route.is_some() && (lease.is_some() || control.intent.manual.is_some()) {
            return None;
        }
        let committed_pulse = match lease {
            Some(lease) => Some(
                CommittedPulse::admit(lease, result.basis.clone(), latest, current_tick).ok()?,
            ),
            None => None,
        };
        let committed_route = match route {
            Some(route) => Some(
                CommittedRoute::admit(
                    route,
                    result.basis,
                    latest,
                    current_tick,
                    control.intent.movement,
                )
                .ok()?,
            ),
            None => None,
        };
        Some(AcceptedPlan {
            decision: pending.decision,
            intent: control.intent,
            hold_ticks: control.hold_ticks,
            next_memory: control.next_memory,
            next_rng: result.rng,
            committed_pulse,
            committed_route,
        })
    }

    pub(in crate::game::rooms) fn commit(&mut self, accepted: AcceptedPlan, actual_intent: Intent) {
        let planted = accepted.intent.manual == Some(grenadiers_core::cpu::ManualIntent::Bomb)
            && actual_intent.manual == accepted.intent.manual;
        self.memory = Some(
            if accepted.intent.manual == Some(grenadiers_core::cpu::ManualIntent::Bomb) && !planted
            {
                UtilityMemory::new(accepted.next_memory.key())
            } else {
                accepted.next_memory
            },
        );
        self.rng = Some(accepted.next_rng);
        self.escaping = planted;
        self.committed_pulse = accepted.committed_pulse;
        self.committed_route_epoch = accepted
            .committed_route
            .as_ref()
            .map(|_| accepted.decision.command_epoch);
        self.committed_route = accepted.committed_route;
        if planted {
            self.cancel();
        }
        if let Some(memory) = self.admission_memory.as_mut() {
            let _ = memory.remember(accepted.decision, actual_intent);
        }
    }

    pub(in crate::game::rooms) const fn is_escaping(&self) -> bool {
        self.escaping
    }

    pub(in crate::game::rooms) fn follow_escape(
        &mut self,
        observation: &Observation,
    ) -> Result<EscapeExecution, PolicyError> {
        if !self.escaping {
            return Ok(EscapeExecution::Completed);
        }
        let execution = grenadiers_core::cpu::policy::follow_checked_escape(
            observation,
            self.memory.as_ref().expect("world memory initialized"),
        )?;
        if let EscapeExecution::Continue(plan) | EscapeExecution::Wait(plan) = execution {
            if !(1..=MAX_HOLD_TICKS).contains(&plan.hold_ticks) || plan.intent.manual.is_some() {
                return Err(PolicyError::InvalidObservation);
            }
            return Ok(execution);
        }
        self.escaping = false;
        self.memory = Some(UtilityMemory::new(
            self.memory
                .as_ref()
                .expect("world memory initialized")
                .key(),
        ));
        Ok(execution)
    }

    pub(in crate::game::rooms) fn commit_escape(&mut self, plan: ControlPlan) {
        self.memory = Some(plan.next_memory);
    }

    pub(in crate::game::rooms) fn cancel(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.live.store(false, Ordering::Release);
        }
    }

    pub(in crate::game::rooms) const fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn plan_now_for_test(
        &mut self,
        observation: &Observation,
        command_epoch: u64,
    ) -> AcceptedPlan {
        self.cancel();
        self.accept_observation(observation);
        let serial = self.request_serial + 1;
        self.request_serial = serial;
        let decision = DecisionId {
            world: observation.parts().world,
            command_epoch,
            observation_tick: observation.parts().tick,
            checked_request_serial: serial,
        };
        let mut rng = self.rng.as_ref().unwrap().clone();
        let plan = grenadiers_core::cpu::policy::plan(
            observation,
            self.memory.as_ref().unwrap(),
            &mut rng,
            &mut grenadiers_core::cpu::WorkBudget::new(256),
        )
        .unwrap();
        AcceptedPlan {
            decision,
            intent: plan.intent,
            hold_ticks: plan.hold_ticks,
            next_memory: plan.next_memory,
            next_rng: rng,
            committed_pulse: None,
            committed_route: None,
        }
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn rng_draws(&self) -> u64 {
        self.rng.as_ref().map_or(0, CpuRng::draws)
    }
}

impl Drop for CpuController {
    fn drop(&mut self) {
        self.cancel();
    }
}

#[path = "controller_observation.rs"]
mod observation;

#[path = "controller_route.rs"]
mod route;

#[path = "controller_pulse.rs"]
mod pulse;

#[cfg(test)]
#[path = "controller_test_support.rs"]
mod test_support;

fn derive_round_seed(mut seed: [u8; 32], physical_round: u64) -> [u8; 32] {
    let round = physical_round.to_le_bytes();
    for (index, value) in seed.iter_mut().enumerate() {
        *value ^= round[index % round.len()].wrapping_add((index as u8).wrapping_mul(0x5d));
        *value = value.rotate_left((index % 7 + 1) as u32);
    }
    seed
}

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "controller_route_tests.rs"]
mod route_tests;
