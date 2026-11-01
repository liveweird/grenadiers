use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, sync_channel},
};

use grenadiers_core::cpu::forecast::ObservedHazardMemory;
use grenadiers_core::cpu::survival::{CommittedPulse, CommittedRoute, PlantEscapeOrigin};
use grenadiers_core::cpu::{
    CpuMemory, CpuRng, DecisionId, Intent, Observation, WorldId, policy::UtilityMemory,
};
use grenadiers_core::game::{SeatId, Tick};
use grenadiers_core::room::CpuProfile;

use self::cadence::CadenceState;
use super::worker::{CpuPlanner, PlanJob, PlanResult};

struct Pending {
    decision: DecisionId,
    receiver: Receiver<PlanResult>,
    live: Arc<AtomicBool>,
    completed: Option<PlanResult>,
    refuge_handoff: Option<grenadiers_core::cpu::survival::TerminalRefugeHandoff>,
}

pub(in crate::game::rooms) struct AcceptedPlan {
    pub(in crate::game::rooms) decision: DecisionId,
    pub(in crate::game::rooms) intent: Intent,
    pub(in crate::game::rooms) hold_ticks: u8,
    next_memory: UtilityMemory,
    next_rng: CpuRng,
    committed_pulse: Option<CommittedPulse>,
    committed_route: Option<CommittedRoute>,
    contact: Option<grenadiers_core::cpu::MobContactLease>,
    certified_manual: Option<grenadiers_core::cpu::ManualIntent>,
    escape_origin: bool,
    ordinary_navigation: bool,
    refuge_handoff: bool,
}

#[derive(Clone, Copy)]
struct StopAdmission {
    tick: Tick,
}

pub(in crate::game::rooms) struct CpuController {
    planner: CpuPlanner,
    cpu_seat: SeatId,
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
    committed_contact: Option<grenadiers_core::cpu::MobContactLease>,
    committed_route_epoch: Option<u64>,
    escape_origin: Option<PlantEscapeOrigin>,
    escape_origin_epoch: Option<u64>,
    profile: CpuProfile,
    cadence: CadenceState,
    strategic_blast: Option<intent::StrategicBlastContext>,
    refuge_attempt: Option<refuge_handoff::AttemptIdentity>,
    #[cfg(test)]
    admission_trace: admission::AdmissionTrace,
}

impl CpuController {
    pub(in crate::game::rooms) fn new(
        planner: CpuPlanner,
        cpu_seat: SeatId,
        room_instance: u64,
        private_seed: [u8; 32],
        profile: CpuProfile,
    ) -> Self {
        Self {
            planner,
            cpu_seat,
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
            committed_contact: None,
            committed_route_epoch: None,
            escape_origin: None,
            escape_origin_epoch: None,
            profile,
            cadence: CadenceState::default(),
            strategic_blast: None,
            refuge_attempt: None,
            #[cfg(test)]
            admission_trace: admission::AdmissionTrace::default(),
        }
    }

    pub(in crate::game::rooms) fn submit(
        &mut self,
        observation: Observation,
        command_epoch: u64,
    ) -> bool {
        if observation.parts().recipient != self.cpu_seat {
            return false;
        }
        self.validate_escape_epoch(command_epoch);
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
        if self.defer_planning(&observation, command_epoch) {
            return false;
        }
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
            policy: self.profile.policy,
            observation,
            memory: *self.memory.as_ref().expect("world memory initialized"),
            hazard_memory: self
                .hazard_memory
                .as_ref()
                .expect("world hazard memory initialized")
                .clone(),
            settled_stop_tick,
            rng: self.rng.as_ref().expect("world rng initialized").clone(),
            escape_origin: self.escape_origin.clone(),
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
            completed: None,
            refuge_handoff: None,
        });
        true
    }

    pub(in crate::game::rooms) fn commit(&mut self, accepted: AcceptedPlan, actual_intent: Intent) {
        self.arm_cadence_after(&accepted, actual_intent);
        let manual_admitted =
            accepted.intent.manual.is_some() && actual_intent.manual == accepted.intent.manual;
        self.memory = Some(
            if accepted.certified_manual.is_none()
                && accepted.intent.manual == Some(grenadiers_core::cpu::ManualIntent::Bomb)
                && !manual_admitted
            {
                UtilityMemory::new(accepted.next_memory.key())
            } else {
                accepted.next_memory
            },
        );
        if accepted.certified_manual == Some(grenadiers_core::cpu::ManualIntent::Bomb)
            && !manual_admitted
            && let Some(memory) = self.memory
        {
            let reason = memory.strategic_intent().map_or(
                grenadiers_core::cpu::policy::StrategicSwitchReason::Initial,
                |intent| intent.last_switch_reason,
            );
            self.memory = Some(memory.mark_strategic_phase(
                grenadiers_core::cpu::policy::StrategicPhase::Approach,
                reason,
                accepted.decision.observation_tick,
            ));
        }
        self.rng = Some(accepted.next_rng);
        self.escaping = accepted.certified_manual.is_none()
            && accepted.intent.manual == Some(grenadiers_core::cpu::ManualIntent::Bomb)
            && manual_admitted;
        self.committed_pulse = accepted.committed_pulse;
        if accepted.escape_origin
            && actual_intent.movement != grenadiers_core::cpu::MovementIntent::Stop
            && self.committed_pulse.is_none()
        {
            self.clear_escape_origin();
        }
        self.committed_route_epoch = accepted
            .committed_route
            .as_ref()
            .map(|_| accepted.decision.command_epoch);
        self.committed_route = accepted.committed_route;
        self.committed_contact = accepted.contact;
        if !manual_admitted {
            match accepted.certified_manual {
                Some(grenadiers_core::cpu::ManualIntent::Bomb) => {
                    let _ = self
                        .committed_route
                        .as_mut()
                        .is_some_and(CommittedRoute::select_absent_plant);
                }
                Some(grenadiers_core::cpu::ManualIntent::Equipment) => {
                    let _ = self
                        .committed_route
                        .as_mut()
                        .is_some_and(CommittedRoute::select_unchanged_punch);
                }
                None => {}
            }
        }
        if self.escaping {
            self.cancel();
        }
        if let Some(memory) = self.admission_memory.as_mut() {
            let _ = memory.remember(accepted.decision, actual_intent);
        }
    }

    pub(in crate::game::rooms) const fn is_escaping(&self) -> bool {
        self.escaping
    }

    pub(in crate::game::rooms) fn cancel(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.live.store(false, Ordering::Release);
        }
    }

    pub(in crate::game::rooms) const fn has_pending(&self) -> bool {
        self.pending.is_some()
    }

    pub(in crate::game::rooms) fn clear_escape_origin(&mut self) {
        self.escape_origin = None;
        self.escape_origin_epoch = None;
    }

    #[cfg(test)]
    pub(in crate::game::rooms) const fn recovery_state(&self) -> (bool, bool) {
        (
            self.escape_origin.is_some(),
            self.escape_origin.is_some() && self.committed_pulse.is_some(),
        )
    }

    pub(in crate::game::rooms) fn validate_escape_epoch(&mut self, command_epoch: u64) {
        if self.escape_origin.is_some() && self.escape_origin_epoch != Some(command_epoch) {
            self.clear_escape_origin();
            self.cancel();
        }
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
            contact: None,
            certified_manual: None,
            escape_origin: false,
            ordinary_navigation: false,
            refuge_handoff: false,
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
#[path = "controller_debug.rs"]
mod debug;

#[path = "controller_observation.rs"]
mod observation;

#[path = "controller_route.rs"]
mod route;

#[path = "controller_admission.rs"]
pub(in crate::game::rooms) mod admission;

#[path = "controller_intent.rs"]
mod intent;

#[path = "controller_pulse.rs"]
mod pulse;

#[path = "controller_cadence.rs"]
mod cadence;

#[path = "controller_contact.rs"]
pub(in crate::game::rooms) mod contact;

#[path = "controller_escape.rs"]
mod escape;

#[path = "controller_seed.rs"]
mod seed;

#[cfg(test)]
#[path = "controller_test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "controller_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "controller_route_tests.rs"]
mod route_tests;

#[cfg(test)]
#[path = "controller_bracelet_tests.rs"]
mod bracelet_tests;

#[cfg(test)]
#[path = "controller_plant_tests.rs"]
mod plant_tests;

#[cfg(test)]
#[path = "controller_recovery_tests.rs"]
mod recovery_tests;

#[cfg(test)]
#[path = "controller_lair_recovery_tests.rs"]
mod lair_recovery_tests;

#[cfg(test)]
#[path = "controller_weapon_tests.rs"]
mod weapon_tests;

#[cfg(test)]
#[path = "controller_multiplayer_tests.rs"]
mod multiplayer_tests;

#[cfg(test)]
#[path = "controller_respawn_tests.rs"]
mod respawn_tests;

#[path = "controller_profile.rs"]
mod profile;

#[path = "controller_refuge_handoff.rs"]
mod refuge_handoff;

#[cfg(test)]
#[path = "controller_refuge_handoff_tests.rs"]
mod refuge_handoff_tests;

#[cfg(test)]
#[path = "refuge_handoff_test_support.rs"]
mod refuge_handoff_test_support;
