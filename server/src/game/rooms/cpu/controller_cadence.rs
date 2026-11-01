use grenadiers_core::cpu::{
    BombKnowledge, Intent, Observation, ObservedActor, WorkBudget, WorldId,
};
use grenadiers_core::game::{EffectKind, OilBarrelState, Tick};
use grenadiers_core::room::CpuProfile;

use super::{AcceptedPlan, CpuController};

const EASY_PAUSE_TICKS: u64 = 12;

#[derive(Clone, Copy)]
struct CadenceKey {
    world: WorldId,
    command_epoch: u64,
}

#[derive(Clone, Copy)]
struct CadenceDeadline {
    key: CadenceKey,
    tick: Tick,
}

#[derive(Default)]
pub(super) struct CadenceState {
    armed: Option<CadenceKey>,
    deadline: Option<CadenceDeadline>,
}

impl CpuController {
    pub(super) fn arm_cadence_after(&mut self, accepted: &AcceptedPlan, actual: Intent) {
        if !matches!(
            self.profile,
            CpuProfile::CURRENT_EASY
                | CpuProfile::MULTIPLAYER_EASY
                | CpuProfile::ENVIRONMENT_EASY
                | CpuProfile::LAIRS_EASY
                | CpuProfile::RESPAWN_LAIRS_EASY
                | CpuProfile::RESPAWN_V24_EASY
                | CpuProfile::RESPAWN_V23_EASY
                | CpuProfile::RESPAWN_V22_EASY
                | CpuProfile::RESPAWN_V21_EASY
                | CpuProfile::SPIKE_PITS_EASY
                | CpuProfile::RELIABLE_V27_EASY
                | CpuProfile::PICKUP_V31_EASY
                | CpuProfile::REFUGE_V33_EASY
                | CpuProfile::FARM_REFUGES_V36_EASY
        ) || !accepted.ordinary_navigation
            || actual.manual.is_some()
            || actual.movement == grenadiers_core::cpu::MovementIntent::Stop
        {
            return;
        }
        self.cadence.armed = Some(CadenceKey {
            world: accepted.decision.world,
            command_epoch: accepted.decision.command_epoch,
        });
        self.cadence.deadline = None;
    }

    pub(super) fn observe_cadence(&mut self, observation: &Observation) {
        if !matches!(
            self.profile,
            CpuProfile::CURRENT_EASY
                | CpuProfile::MULTIPLAYER_EASY
                | CpuProfile::ENVIRONMENT_EASY
                | CpuProfile::LAIRS_EASY
                | CpuProfile::RESPAWN_LAIRS_EASY
                | CpuProfile::RESPAWN_V24_EASY
                | CpuProfile::RESPAWN_V23_EASY
                | CpuProfile::RESPAWN_V22_EASY
                | CpuProfile::RESPAWN_V21_EASY
                | CpuProfile::SPIKE_PITS_EASY
                | CpuProfile::RELIABLE_V27_EASY
                | CpuProfile::PICKUP_V31_EASY
                | CpuProfile::REFUGE_V33_EASY
                | CpuProfile::FARM_REFUGES_V36_EASY
        ) || !is_clear_for_easy(observation)
        {
            self.cadence = CadenceState::default();
            return;
        }
        let world = observation.parts().world;
        if self.cadence.armed.is_some_and(|key| key.world != world)
            || self
                .cadence
                .deadline
                .is_some_and(|deadline| deadline.key.world != world)
        {
            self.cadence = CadenceState::default();
            return;
        }
        if self.escaping
            || self.escape_origin.is_some()
            || self.committed_route.is_some()
            || self.committed_pulse.is_some()
        {
            self.cadence = CadenceState::default();
            return;
        }
        let Some(stopped) = self.settled_stop_tick else {
            return;
        };
        let Some(armed) = self.cadence.armed.take() else {
            return;
        };
        let Some(deadline) = stopped.0.checked_add(EASY_PAUSE_TICKS).map(Tick) else {
            self.cadence = CadenceState::default();
            return;
        };
        self.cadence.deadline = Some(CadenceDeadline {
            key: armed,
            tick: deadline,
        });
    }

    pub(super) fn defer_planning(&mut self, observation: &Observation, command_epoch: u64) -> bool {
        let Some(deadline) = self.cadence.deadline else {
            return false;
        };
        if deadline.key.world != observation.parts().world
            || deadline.key.command_epoch != command_epoch
            || !is_clear_for_easy(observation)
        {
            self.cadence = CadenceState::default();
            return false;
        }
        if observation.parts().tick < deadline.tick {
            return true;
        }
        self.cadence.deadline = None;
        false
    }
}

pub(super) fn is_clear_for_easy(observation: &Observation) -> bool {
    let parts = observation.parts();
    if observation.spike_pits().is_some()
        || !matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        || !parts.flames.is_empty()
        || parts
            .oil
            .iter()
            .any(|oil| oil.state == OilBarrelState::Burning)
    {
        return false;
    }
    if (observation.lairs().is_some() || observation.respawn_lairs().is_some())
        && parts
            .tick
            .0
            .checked_add(EASY_PAUSE_TICKS)
            .map(Tick)
            .is_none_or(|through| {
                grenadiers_core::cpu::has_urgent_lair_threat(
                    observation,
                    through,
                    &mut WorkBudget::new(65_536),
                )
            })
    {
        return false;
    }
    parts.actors.iter().any(|actor| {
        matches!(actor,
            ObservedActor::Visible(actor)
                if actor.seat == parts.recipient
                    && actor.alive
                    && !actor.voluntary_actions_frozen
                    && !actor.effects.iter().any(|effect| matches!(
                        effect.kind,
                        EffectKind::Diarrhea
                            | EffectKind::SlowFoot
                            | EffectKind::ReverseControls
                    ))
        )
    })
}

#[cfg(test)]
mod tests {
    use grenadiers_core::cpu::{DecisionId, MovementIntent, policy::UtilityMemory};
    use grenadiers_core::game::{CellId, Direction};

    use super::*;
    use crate::game::rooms::cpu::CpuPlanner;

    #[test]
    fn easy_pause_is_bounded_and_repeated_observations_do_not_restart_it() {
        let mut controller = armed_controller(CpuProfile::CURRENT_EASY);
        controller.on_stop_admitted(Tick(20));

        for tick in 21..33 {
            let observation = observation_at(tick);
            assert!(controller.accept_observation(&observation));
            assert!(!controller.submit(observation, 4));
            assert!(!controller.has_pending());
        }

        let due = observation_at(33);
        assert!(controller.accept_observation(&due));
        assert!(controller.submit(due, 4));
    }

    #[test]
    fn standard_replans_immediately_after_the_same_confirmed_stop() {
        let mut controller = armed_controller(CpuProfile::CURRENT);
        controller.on_stop_admitted(Tick(20));
        let stopped = observation_at(21);

        assert!(controller.accept_observation(&stopped));
        assert!(controller.submit(stopped, 4));
    }

    #[test]
    fn newly_visible_danger_bypasses_and_clears_the_easy_pause() {
        let mut controller = armed_controller(CpuProfile::CURRENT_EASY);
        controller.on_stop_admitted(Tick(20));
        let stopped = observation_at(21);
        assert!(controller.accept_observation(&stopped));
        assert!(!controller.submit(stopped, 4));

        let mut parts = observation_at(22).into_parts();
        parts.flames.push(grenadiers_core::cpu::ObservedFlame {
            cell: CellId::new(6, 6),
            remaining_ticks: 2,
        });
        let danger = Observation::new(parts).unwrap();
        assert!(controller.accept_observation(&danger));
        assert!(controller.submit(danger, 4));
    }

    #[test]
    fn rules_seven_easy_pause_is_bounded_after_confirmed_clear_stop() {
        let mut controller = armed_controller(CpuProfile::ENVIRONMENT_EASY);
        controller.on_stop_admitted(Tick(20));

        for tick in 21..33 {
            let observation = environment_observation_at(tick);
            assert!(controller.accept_observation(&observation));
            assert!(!controller.submit(observation, 4));
            assert!(!controller.has_pending());
        }

        let due = environment_observation_at(33);
        assert!(controller.accept_observation(&due));
        assert!(controller.submit(due, 4));
    }

    #[test]
    fn rules_seven_hazard_interrupts_environment_easy_pause() {
        let mut controller = armed_controller(CpuProfile::ENVIRONMENT_EASY);
        controller.on_stop_admitted(Tick(20));
        let stopped = environment_observation_at(21);
        assert!(controller.accept_observation(&stopped));
        assert!(!controller.submit(stopped, 4));

        let danger = environment_observation_with_flame(22);
        assert!(controller.accept_observation(&danger));
        assert!(controller.submit(danger, 4));
    }

    #[test]
    fn rules_seven_single_cloud_environment_easy_uses_the_same_pause() {
        let basis = environment_observation(20, false, true);
        let mut controller = armed_controller_with_basis(CpuProfile::ENVIRONMENT_EASY, basis);
        controller.on_stop_admitted(Tick(20));
        let stopped = environment_observation(21, false, true);

        assert!(controller.accept_observation(&stopped));
        assert!(!controller.submit(stopped, 4));
        assert!(!controller.has_pending());
    }

    #[test]
    fn unsupported_control_effects_bypass_and_clear_the_easy_pause() {
        for kind in [EffectKind::SlowFoot, EffectKind::ReverseControls] {
            let mut controller = armed_controller(CpuProfile::CURRENT_EASY);
            controller.on_stop_admitted(Tick(20));
            let stopped = observation_at(21);
            assert!(controller.accept_observation(&stopped));
            assert!(!controller.submit(stopped, 4));

            let mut parts = observation_at(22).into_parts();
            let ObservedActor::Visible(cpu) = &mut parts.actors[1] else {
                unreachable!()
            };
            cpu.effects.push(grenadiers_core::cpu::ObservedEffect {
                kind,
                remaining_ticks: 10,
            });
            let changed = Observation::new(parts).unwrap();
            assert!(controller.accept_observation(&changed));
            assert!(controller.submit(changed, 4), "effect {kind:?}");
        }
    }

    fn armed_controller(profile: CpuProfile) -> CpuController {
        let basis = if profile == CpuProfile::ENVIRONMENT_EASY {
            environment_observation_at(20)
        } else {
            observation_at(20)
        };
        armed_controller_with_basis(profile, basis)
    }

    fn armed_controller_with_basis(profile: CpuProfile, basis: Observation) -> CpuController {
        let mut controller = CpuController::new(
            CpuPlanner::without_workers(),
            grenadiers_core::game::SeatId::Two,
            7,
            [5; 32],
            profile,
        );
        assert!(controller.accept_observation(&basis));
        controller.settled_stop_tick = Some(basis.parts().tick);
        let key = controller.memory.as_ref().unwrap().key();
        let intent = Intent::moving(Direction::West);
        let accepted = AcceptedPlan {
            decision: DecisionId {
                world: basis.parts().world,
                command_epoch: 4,
                observation_tick: basis.parts().tick,
                checked_request_serial: 1,
            },
            intent,
            hold_ticks: 1,
            next_memory: UtilityMemory::new(key),
            next_rng: controller.rng.as_ref().unwrap().clone(),
            committed_pulse: None,
            committed_route: None,
            contact: None,
            certified_manual: None,
            escape_origin: false,
            ordinary_navigation: true,
            refuge_handoff: false,
        };
        controller.on_movement_admitted();
        controller.commit(accepted, intent);
        assert_eq!(intent.movement, MovementIntent::Direction(Direction::West));
        controller
    }

    fn observation_at(tick: u64) -> Observation {
        let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
        parts.tick = Tick(tick);
        Observation::new(parts).unwrap()
    }

    fn environment_observation_at(tick: u64) -> Observation {
        environment_observation(tick, false, false)
    }

    fn environment_observation_with_flame(tick: u64) -> Observation {
        environment_observation(tick, true, false)
    }

    fn environment_observation(tick: u64, flame: bool, single_cloud: bool) -> Observation {
        let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
        parts.tick = Tick(tick);
        parts.rules.version = 7;
        if flame {
            parts.flames.push(grenadiers_core::cpu::ObservedFlame {
                cell: CellId::new(6, 6),
                remaining_ticks: 2,
            });
        }
        let timers = parts
            .actors
            .iter()
            .map(|actor| match actor {
                ObservedActor::Visible(_) => Some(0),
                ObservedActor::Hidden { .. } => None,
            })
            .collect();
        Observation::new_rules_v7(
            parts,
            grenadiers_core::cpu::ObservedEnvironment::new(
                if single_cloud {
                    grenadiers_core::game::StormCloudConfig::Single
                } else {
                    grenadiers_core::game::StormCloudConfig::Disabled
                },
                single_cloud.then_some(grenadiers_core::game::StormCloudView {
                    position: grenadiers_core::game::Position::center(CellId::new(7, 6)),
                    direction: Direction::East,
                    leg_remaining_ticks: 64,
                }),
                timers,
            ),
        )
        .unwrap()
    }
}
