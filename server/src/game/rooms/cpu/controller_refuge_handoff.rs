use std::sync::{Arc, atomic::AtomicBool, mpsc::sync_channel};

use grenadiers_core::cpu::policy::{StrategicPhase, StrategicSwitchReason};
use grenadiers_core::cpu::survival::TerminalRefugeHandoff;
use grenadiers_core::cpu::{DecisionId, Observation};
use grenadiers_core::game::{EntityId, Tick};

use super::{AcceptedPlan, CpuController, Pending};
use crate::game::rooms::cpu::worker::PlanJob;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AttemptIdentity {
    route_basis_tick: Tick,
    route_first_update: Tick,
    bomb_id: EntityId,
}

impl AttemptIdentity {
    fn from_handoff(handoff: &TerminalRefugeHandoff) -> Self {
        let (route_basis_tick, route_first_update) = handoff.route_identity();
        Self {
            route_basis_tick,
            route_first_update,
            bomb_id: handoff.bomb_id(),
        }
    }
}

impl CpuController {
    /// Enqueues one best-effort replacement attempt for a confirmed terminal
    /// plant refuge. The installed route, live memory, and live RNG are only
    /// read here; they remain authoritative until owner commit.
    pub(in crate::game::rooms) fn submit_refuge_handoff(
        &mut self,
        observation: Observation,
        command_epoch: u64,
    ) -> bool {
        if self.profile.policy.version() != 30 {
            return false;
        }
        self.enqueue_refuge_handoff(observation, command_epoch)
    }

    fn enqueue_refuge_handoff(&mut self, observation: Observation, command_epoch: u64) -> bool {
        if self.pending.is_some()
            || self.escaping
            || self.committed_pulse.is_some()
            || self.escape_origin.is_some()
            || self.world != Some(observation.parts().world)
            || self.committed_route_epoch != Some(command_epoch)
        {
            return false;
        }
        let settled_stop_tick = match self.settled_stop_tick {
            Some(tick) => tick,
            None => return false,
        };
        let handoff =
            match self.committed_route.as_ref().and_then(|route| {
                route.terminal_refuge_handoff(&observation, Some(settled_stop_tick))
            }) {
                Some(handoff) => handoff,
                None => return false,
            };
        let identity = AttemptIdentity::from_handoff(&handoff);
        if self.refuge_attempt == Some(identity) {
            return false;
        }
        let Some(serial) = self.request_serial.checked_add(1) else {
            return false;
        };
        let Some(memory) = self.memory.and_then(|memory| {
            memory.strategic_intent().map(|_| {
                memory
                    .mark_strategic_phase(
                        StrategicPhase::Reengage,
                        StrategicSwitchReason::Resumed,
                        observation.parts().tick,
                    )
                    .without_execution_authority()
            })
        }) else {
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
            memory,
            hazard_memory: self
                .hazard_memory
                .as_ref()
                .expect("world hazard memory initialized")
                .clone(),
            settled_stop_tick,
            rng: self.rng.as_ref().expect("world rng initialized").clone(),
            escape_origin: None,
            live: Arc::clone(&live),
            result: sender,
        };
        if !self.planner.try_submit(job) {
            return false;
        }
        self.request_serial = serial;
        self.refuge_attempt = Some(identity);
        self.pending = Some(Pending {
            decision,
            receiver,
            live,
            completed: None,
            refuge_handoff: Some(handoff),
        });
        true
    }

    #[cfg(test)]
    pub(in crate::game::rooms) fn submit_refuge_handoff_for_test(
        &mut self,
        observation: Observation,
        command_epoch: u64,
    ) -> bool {
        self.enqueue_refuge_handoff(observation, command_epoch)
    }

    pub(super) fn refuge_handoff_still_installed(
        &self,
        expected: &TerminalRefugeHandoff,
        latest: &Observation,
    ) -> bool {
        let Some(settled) = self.settled_stop_tick else {
            return false;
        };
        self.committed_route
            .as_ref()
            .and_then(|route| route.terminal_refuge_handoff(latest, Some(settled)))
            .is_some_and(|current| {
                AttemptIdentity::from_handoff(&current) == AttemptIdentity::from_handoff(expected)
                    && current.old_survive_until() == expected.old_survive_until()
            })
    }
}

impl AcceptedPlan {
    pub(in crate::game::rooms) const fn is_refuge_handoff(&self) -> bool {
        self.refuge_handoff
    }
}
