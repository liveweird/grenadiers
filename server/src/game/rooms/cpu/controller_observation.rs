use grenadiers_core::cpu::{
    CpuMemory, CpuRng, MemoryKey, Observation, ObservedActor, WorldId,
    forecast::ObservedHazardMemory, policy::UtilityMemory,
};
use grenadiers_core::game::{SeatId, Tick};

use super::{CpuController, StopAdmission, derive_round_seed};

impl CpuController {
    pub(in crate::game::rooms) fn reset_world(&mut self, physical_round: u64) {
        self.cancel();
        let world = WorldId {
            room_instance: self.room_instance,
            physical_round,
        };
        let key = MemoryKey {
            world,
            cpu_seat: SeatId::Two,
        };
        self.world = Some(world);
        self.admission_memory = CpuMemory::new(key, 64).ok();
        self.memory = Some(UtilityMemory::new(key));
        self.hazard_memory = Some(ObservedHazardMemory::new(key));
        self.rng = Some(CpuRng::from_seed(
            key,
            derive_round_seed(self.private_seed, physical_round),
        ));
        self.escaping = false;
        self.last_observation_tick = None;
        self.latest_observation = None;
        self.stop_admission = None;
        self.settled_stop_tick = None;
        self.committed_pulse = None;
        self.committed_route = None;
        self.committed_route_epoch = None;
    }

    pub(in crate::game::rooms) fn accept_observation(&mut self, observation: &Observation) -> bool {
        if self.world != Some(observation.parts().world) {
            self.reset_world(observation.parts().world.physical_round);
        }
        if self
            .last_observation_tick
            .is_some_and(|tick| tick >= observation.parts().tick)
        {
            return false;
        }
        let _ = self
            .hazard_memory
            .as_mut()
            .expect("world hazard memory initialized")
            .observe(observation);
        self.last_observation_tick = Some(observation.parts().tick);
        self.confirm_stop(observation);
        self.latest_observation = Some(observation.clone());
        true
    }

    pub(in crate::game::rooms) const fn needs_stop_confirmation(&self) -> bool {
        self.stop_admission.is_none() && self.settled_stop_tick.is_none()
    }

    pub(in crate::game::rooms) fn on_stop_admitted(&mut self, tick: Tick) {
        if self.latest_observation.is_none() {
            return;
        }
        self.stop_admission = Some(StopAdmission { tick });
        self.settled_stop_tick = None;
    }

    pub(in crate::game::rooms) fn on_movement_admitted(&mut self) {
        self.stop_admission = None;
        self.settled_stop_tick = None;
    }

    fn confirm_stop(&mut self, observation: &Observation) {
        let Some(admission) = self.stop_admission else {
            return;
        };
        let Some(first_stopped_tick) = admission.tick.0.checked_add(1).map(Tick) else {
            return;
        };
        if observation.parts().tick < first_stopped_tick {
            return;
        }
        let Some(actor) = cpu_actor(observation) else {
            return;
        };
        if actor.alive && !actor.voluntary_actions_frozen {
            self.settled_stop_tick = Some(observation.parts().tick);
            self.stop_admission = None;
        }
    }
}

fn cpu_actor(observation: &Observation) -> Option<&grenadiers_core::cpu::VisibleActor> {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
}
