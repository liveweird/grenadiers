use std::sync::{Arc, atomic::AtomicBool, mpsc::sync_channel};

use grenadiers_core::cpu::{DecisionId, Observation, survival::SurvivalPlan};

use super::{AcceptedPlan, CpuController, Pending};
use crate::game::rooms::cpu::worker::PlanResult;

pub(super) fn key_for(controller: &CpuController) -> grenadiers_core::cpu::MemoryKey {
    grenadiers_core::cpu::MemoryKey {
        world: controller.world.unwrap(),
        cpu_seat: grenadiers_core::game::SeatId::Two,
    }
}

impl CpuController {
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

    pub(in crate::game::rooms) fn rng_draws(&self) -> u64 {
        self.rng
            .as_ref()
            .map_or(0, grenadiers_core::cpu::CpuRng::draws)
    }

    pub(in crate::game::rooms) fn inject_plan_result_for_test(
        &mut self,
        basis: Observation,
        command_epoch: u64,
        plan: SurvivalPlan,
    ) {
        self.cancel();
        let serial = self.request_serial + 1;
        self.request_serial = serial;
        let decision = DecisionId {
            world: basis.parts().world,
            command_epoch,
            observation_tick: basis.parts().tick,
            checked_request_serial: serial,
        };
        let (sender, receiver) = sync_channel(1);
        sender
            .send(PlanResult {
                plan,
                contact: None,
                basis,
                rng: self.rng.as_ref().expect("world rng initialized").clone(),
                escape_origin: None,
            })
            .expect("test result receiver is live");
        self.pending = Some(Pending {
            decision,
            receiver,
            live: Arc::new(AtomicBool::new(true)),
            completed: None,
            refuge_handoff: None,
        });
    }
}

impl AcceptedPlan {
    pub(in crate::game::rooms) fn has_certificate_for_test(&self) -> bool {
        self.committed_pulse.is_some() || self.committed_route.is_some()
    }
}
