use super::{CpuController, StopAdmission, seed::derive_round_seed};
use grenadiers_core::cpu::{
    CpuMemory, CpuRng, MemoryKey, Observation, ObservedActor, WorldId,
    forecast::ObservedHazardMemory, policy::UtilityMemory,
};
use grenadiers_core::game::{SeatId, Tick};
impl CpuController {
    pub(in crate::game::rooms) const fn seat(&self) -> SeatId {
        self.cpu_seat
    }
    pub(in crate::game::rooms) const fn profile(&self) -> grenadiers_core::room::CpuProfile {
        self.profile
    }

    pub(in crate::game::rooms) const fn room_instance(&self) -> u64 {
        self.room_instance
    }

    pub(in crate::game::rooms) fn reset_world(&mut self, physical_round: u64) {
        self.cancel();
        let world = WorldId {
            room_instance: self.room_instance,
            physical_round,
        };
        let key = MemoryKey {
            world,
            cpu_seat: self.cpu_seat,
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
        self.committed_contact = None;
        self.committed_route_epoch = None;
        self.escape_origin = None;
        self.escape_origin_epoch = None;
        self.cadence = super::CadenceState::default();
        self.strategic_blast = None;
        #[cfg(test)]
        {
            self.admission_trace = super::admission::AdmissionTrace::default();
        }
    }

    pub(in crate::game::rooms) fn accept_observation(&mut self, observation: &Observation) -> bool {
        if observation.parts().recipient != self.cpu_seat
            || observation.parts().world.room_instance != self.room_instance
            || !self.profile_supports(observation.parts().rules.version)
        {
            return false;
        }
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
        self.observe_strategic_intent(observation);
        self.last_observation_tick = Some(observation.parts().tick);
        self.confirm_stop(observation);
        self.observe_cadence(observation);
        self.retain_escape_origin(observation);
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
            // A scheduled route Stop is an intermediate handoff, not the
            // end of the proof that guards its remaining movement.
            if self.committed_route.is_none() && self.committed_pulse.is_none() {
                self.committed_contact = None;
            }
        }
    }

    fn retain_escape_origin(&mut self, observation: &Observation) {
        let Some(origin) = self.escape_origin.as_ref() else {
            return;
        };
        let retained = if let Some(pulse) = self.committed_pulse.as_ref() {
            pulse.retain_escape_origin(origin, observation, observation.parts().tick)
        } else if self.settled_stop_tick.is_some() {
            origin.retain_stopped(observation)
        } else {
            None
        };
        if retained.is_some() {
            self.escape_origin = retained;
        } else {
            self.clear_escape_origin();
            self.cancel();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::rooms::cpu::{tests::observation_for, worker::CpuPlanner};

    fn controller_for(cpu_seat: SeatId) -> CpuController {
        let mut controller = CpuController::new(
            CpuPlanner::without_workers(),
            cpu_seat,
            7,
            [5; 32],
            grenadiers_core::room::CpuProfile::CURRENT,
        );
        controller.reset_world(3);
        controller
    }

    #[test]
    fn controller_identity_keys_world_memory_for_either_existing_seat() {
        for seat in SeatId::DUEL {
            let controller = controller_for(seat);
            assert_eq!(controller.seat(), seat);
            assert_eq!(controller.memory.as_ref().unwrap().key().cpu_seat, seat);
            assert_eq!(
                controller.admission_memory.as_ref().unwrap().key().cpu_seat,
                seat
            );
            assert_eq!(controller.rng.as_ref().unwrap().key().cpu_seat, seat);
        }
    }

    #[test]
    fn foreign_recipient_cannot_mutate_or_submit_through_controller() {
        let mut controller = CpuController::new(
            CpuPlanner::without_workers(),
            SeatId::One,
            7,
            [5; 32],
            grenadiers_core::room::CpuProfile::CURRENT,
        );
        let own = observation_for(SeatId::One);
        assert!(controller.accept_observation(&own));
        assert_eq!(controller.world, Some(own.parts().world));
        assert_eq!(
            controller.memory.as_ref().unwrap().key().cpu_seat,
            SeatId::One
        );

        let mut foreign_parts = observation_for(SeatId::Two).into_parts();
        foreign_parts.world.physical_round += 1;
        foreign_parts.tick = Tick(21);
        let foreign = Observation::new(foreign_parts).unwrap();
        assert!(!controller.accept_observation(&foreign));
        assert_eq!(controller.world, Some(own.parts().world));
        assert_eq!(
            controller.latest_observation.as_ref().unwrap().parts().tick,
            Tick(20)
        );
        assert!(!controller.submit(foreign, 4));
        assert_eq!(controller.request_serial, 0);
        assert_eq!(controller.rng_draws(), 0);
    }

    #[test]
    fn foreign_room_world_cannot_replace_controller_memory() {
        let mut controller = controller_for(SeatId::One);
        let own = observation_for(SeatId::One);
        assert!(controller.accept_observation(&own));
        let mut foreign_parts = own.clone().into_parts();
        foreign_parts.world.room_instance = 8;
        foreign_parts.world.physical_round += 1;
        foreign_parts.tick = Tick(21);
        let foreign = Observation::new(foreign_parts).unwrap();

        assert!(!controller.accept_observation(&foreign));
        assert_eq!(controller.world, Some(own.parts().world));
        assert_eq!(
            controller.latest_observation.as_ref().unwrap().parts().tick,
            Tick(20)
        );
    }

    #[test]
    fn only_cpu18_profiles_accept_rules_seven_observations() {
        let mut parts = observation_for(SeatId::One).into_parts();
        parts.rules.version = 7;
        let timers = parts
            .actors
            .iter()
            .map(|actor| match actor {
                ObservedActor::Visible(_) => Some(0),
                ObservedActor::Hidden { .. } => None,
            })
            .collect();
        let observed = Observation::new_rules_v7(
            parts,
            grenadiers_core::cpu::ObservedEnvironment::new(
                grenadiers_core::game::StormCloudConfig::Disabled,
                None,
                timers,
            ),
        )
        .unwrap();
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::One,
                7,
                [5; 32],
                profile,
            )
        };

        let mut environment = controller(grenadiers_core::room::CpuProfile::ENVIRONMENT_STANDARD);
        assert!(environment.accept_observation(&observed));
        let mut legacy = controller(grenadiers_core::room::CpuProfile::MULTIPLAYER_STANDARD);
        assert!(!legacy.accept_observation(&observed));
    }

    #[test]
    fn only_cpu21_profiles_accept_rules_nine_observations() {
        let mut parts = observation_for(SeatId::One).into_parts();
        parts.rules.version = 9;
        let timers = parts
            .actors
            .iter()
            .map(|actor| match actor {
                ObservedActor::Visible(_) => Some(0),
                ObservedActor::Hidden { .. } => None,
            })
            .collect();
        let observed = Observation::new_rules_v9(
            parts,
            grenadiers_core::cpu::ObservedEnvironment::new(
                grenadiers_core::game::StormCloudConfig::Disabled,
                None,
                timers,
            ),
            None,
        )
        .unwrap();
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::One,
                7,
                [5; 32],
                profile,
            )
        };
        let mut respawn = controller(grenadiers_core::room::CpuProfile::RESPAWN_LAIRS_STANDARD);
        assert!(respawn.accept_observation(&observed));
        let mut legacy = controller(grenadiers_core::room::CpuProfile::LAIRS_STANDARD);
        assert!(!legacy.accept_observation(&observed));
    }

    #[test]
    fn only_explicit_reliable_profiles_accept_rules10_observations() {
        let mut parts = observation_for(SeatId::One).into_parts();
        parts.rules.version = 10;
        let timers = parts
            .actors
            .iter()
            .map(|actor| match actor {
                ObservedActor::Visible(_) => Some(0),
                ObservedActor::Hidden { .. } => None,
            })
            .collect();
        let observed = Observation::new_rules_v10(
            parts,
            grenadiers_core::cpu::ObservedEnvironment::new(
                grenadiers_core::game::StormCloudConfig::Disabled,
                None,
                timers,
            ),
            None,
            None,
        )
        .unwrap();
        let controller = |profile| {
            CpuController::new(
                CpuPlanner::without_workers(),
                SeatId::One,
                7,
                [5; 32],
                profile,
            )
        };
        for profile in [
            grenadiers_core::room::CpuProfile::SPIKE_PITS_STANDARD,
            grenadiers_core::room::CpuProfile::RELIABLE_V27_STANDARD,
            grenadiers_core::room::CpuProfile::RELIABLE_V27_EASY,
        ] {
            assert!(controller(profile).accept_observation(&observed));
        }
        let mut live = controller(grenadiers_core::room::CpuProfile::RESPAWN_LAIRS_STANDARD);
        assert!(!live.accept_observation(&observed));
    }
}
