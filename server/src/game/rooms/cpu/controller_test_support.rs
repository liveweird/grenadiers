use std::sync::{Arc, atomic::AtomicBool, mpsc::sync_channel};

use grenadiers_core::cpu::{DecisionId, Observation, survival::SurvivalPlan};

use super::{CpuController, Pending};
use crate::game::rooms::cpu::worker::PlanResult;

impl CpuController {
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
        });
    }
}
