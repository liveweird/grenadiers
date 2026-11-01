use super::*;
use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{ControlPlan, StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, EntityId, PickupKind, Tick};

pub(super) fn scene() -> (Observation, UtilityMemory, ObservedHazardMemory) {
    let observation = crate::cpu::lair_contact::v139_teleporter_fixture::captured_tick_6121();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: observation.parts().recipient,
    };
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    (observation, UtilityMemory::new(key), history)
}

pub(super) fn fallback(memory: UtilityMemory) -> LairSurvivalPlan {
    LairSurvivalPlan {
        survival: super::super::SurvivalPlan {
            control: ControlPlan {
                intent: crate::cpu::Intent::moving(Direction::South),
                hold_ticks: 7,
                next_memory: memory,
            },
            lease: None,
            execution: None,
        },
        contact: None,
    }
}

pub(super) fn unrelated_origin() -> PlantEscapeOrigin {
    let basis = crate::cpu::routing::tests::observation(Vec::new(), Vec::new());
    let certificate = crate::cpu::routing::plan_ordinary_plant(
        &basis,
        &ObservedHazardMemory::new(crate::cpu::routing::tests::key()),
        crate::cpu::routing::PlantRequest {
            execution_tick: Tick(7),
            settled_stop_tick: Tick(0),
            first_movement: MovementIntent::Direction(Direction::East),
            limits: crate::cpu::routing::RouteLimits::default(),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let current = crate::cpu::routing::plant_tests::current_scene(
        &basis,
        &certificate,
        Tick(7),
        Some((EntityId(700), certificate.placement().range)),
    );
    PlantEscapeOrigin::new(basis, certificate, EntityId(700), &current)
        .expect("fixture should provide a valid unrelated origin")
}

pub(super) fn assert_fallback(choice: TeleporterRecoveryChoice, memory: UtilityMemory) {
    let TeleporterRecoveryChoice::Ordinary(plan) = choice else {
        panic!("expected unchanged ordinary fallback")
    };
    assert_eq!(plan.survival.control, fallback(memory).survival.control);
    assert!(plan.survival.lease.is_none());
    assert!(plan.survival.execution.is_none());
    assert!(plan.contact.is_none());
}

#[test]
fn captured_scene_selects_an_inactive_progressing_approach() {
    let (observation, memory, history) = scene();
    let mut work = WorkBudget::new(65_536);
    let choice = select_teleporter_recovery(
        &observation,
        &history,
        observation.parts().tick,
        &memory,
        None,
        fallback(memory),
        &mut work,
    );
    let TeleporterRecoveryChoice::Step(plan) = choice else {
        panic!("captured scene should select a step")
    };
    assert_eq!(
        plan.step.phase(),
        crate::cpu::routing::TeleporterStepPhase::Approach
    );
    assert_eq!(
        plan.next_memory.strategic_intent().unwrap().objective,
        StrategicObjectiveKind::Collect
    );
    assert_eq!(
        plan.next_memory.strategic_intent().unwrap().target,
        StrategicTarget::Pickup {
            cell: crate::game::CellId::new(3, 9),
            kind: PickupKind::Teleporter,
        }
    );
    assert!(work.spent() <= TELEPORTER_RECOVERY_CAP);
}

#[test]
fn old_continuous_stop_clock_is_retained_in_certificate() {
    let (observation, memory, history) = scene();
    let old_stop = Tick(observation.parts().tick.0 - 20);
    let choice = select_teleporter_recovery(
        &observation,
        &history,
        old_stop,
        &memory,
        None,
        fallback(memory),
        &mut WorkBudget::new(65_536),
    );
    let TeleporterRecoveryChoice::Step(plan) = choice else {
        panic!("older proven stop should remain eligible")
    };
    assert_eq!(plan.step.settled_stop_tick(), old_stop);
}

#[test]
fn origin_and_exhausted_allowance_return_exact_fallback() {
    let (observation, memory, history) = scene();
    let origin = unrelated_origin();
    let mut origin_work = WorkBudget::new(65_536);
    assert_fallback(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            Some(&origin),
            fallback(memory),
            &mut origin_work,
        ),
        memory,
    );
    assert_eq!(origin_work.spent(), 0);

    let mut exhausted = WorkBudget::new(1);
    exhausted.try_take(1).unwrap();
    assert_fallback(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut exhausted,
        ),
        memory,
    );
    assert_eq!(exhausted.spent(), 1);
}

#[test]
fn missing_target_returns_exact_fallback_after_charged_search() {
    let (observation, memory, history) = scene();
    let observation = observation
        .map_parts(|parts| {
            parts
                .pickups
                .retain(|pickup| pickup.kind != PickupKind::Teleporter);
        })
        .unwrap();
    let mut work = WorkBudget::new(65_536);
    assert_fallback(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut work,
        ),
        memory,
    );
    assert!(work.spent() > 0);
}

#[test]
fn counterfactual_near_edge_pose_selects_collect() {
    let (observation, memory, history) = scene();
    let observation = observation
        .map_parts(|parts| {
            let crate::cpu::ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = crate::game::Position {
                x: 3_584,
                y: 10_250,
            };
            actor.heading = Direction::North;
        })
        .unwrap();
    let choice = select_teleporter_recovery(
        &observation,
        &history,
        observation.parts().tick,
        &memory,
        None,
        fallback(memory),
        &mut WorkBudget::new(65_536),
    );
    let TeleporterRecoveryChoice::Step(plan) = choice else {
        panic!("near-edge pose should select Collect")
    };
    assert_eq!(
        plan.step.phase(),
        crate::cpu::routing::TeleporterStepPhase::Collect
    );
    assert_eq!(
        plan.step.movement(),
        MovementIntent::Direction(Direction::North)
    );
}

#[test]
fn two_candidate_allowances_are_frozen_without_reclaiming_the_first_remainder() {
    assert_eq!(candidate_allowances(11, 2), [5, 6]);
    assert_eq!(candidate_allowances(11, 1), [11, 0]);

    let (observation, memory, history) = scene();
    let observation = observation
        .map_parts(|parts| {
            let crate::cpu::ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = crate::game::Position {
                x: 3_584,
                y: 10_250,
            };
            actor.heading = Direction::North;
        })
        .unwrap();
    let mut work = WorkBudget::new(12_000);
    assert_fallback(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut work,
        ),
        memory,
    );
    assert_eq!(work.spent(), 11_994);
}

#[test]
fn ordinary_adjacent_center_selects_an_approach_toward_the_pickup() {
    let (basis, memory, _) = scene();
    let guidance =
        crate::cpu::policy::teleporter_guidance(&basis, &memory, &mut WorkBudget::new(32_768))
            .unwrap();
    let observation = basis
        .map_parts(|parts| {
            let crate::cpu::ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                unreachable!()
            };
            actor.position = crate::game::Position::center(guidance.approach);
        })
        .unwrap();
    let mut history = ObservedHazardMemory::new(memory.key());
    history.observe(&observation).unwrap();
    let selected = select_teleporter_recovery(
        &observation,
        &history,
        observation.parts().tick,
        &guidance.next_memory,
        None,
        fallback(memory),
        &mut WorkBudget::new(65_536),
    );
    let TeleporterRecoveryChoice::Step(plan) = selected else {
        panic!("adjacent center should begin the target-specific approach")
    };
    assert_eq!(
        plan.step.phase(),
        crate::cpu::routing::TeleporterStepPhase::Approach
    );
    let direction = if guidance.approach.column < guidance.target.column {
        Direction::East
    } else if guidance.approach.column > guidance.target.column {
        Direction::West
    } else if guidance.approach.row < guidance.target.row {
        Direction::South
    } else {
        Direction::North
    };
    assert_eq!(plan.step.movement(), MovementIntent::Direction(direction));
}

#[test]
fn exact_measured_allowance_succeeds_and_one_short_preserves_fallback() {
    let (observation, memory, history) = scene();
    let mut measured = WorkBudget::new(65_536);
    assert!(matches!(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut measured,
        ),
        TeleporterRecoveryChoice::Step(_)
    ));
    let exact = measured.spent();
    assert!(exact < TELEPORTER_RECOVERY_CAP);

    let mut exact_work = WorkBudget::new(exact);
    assert!(matches!(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut exact_work,
        ),
        TeleporterRecoveryChoice::Step(_)
    ));
    assert_eq!(exact_work.spent(), exact);

    let mut short = WorkBudget::new(exact - 1);
    assert_fallback(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut short,
        ),
        memory,
    );
    assert!(short.spent() > 0);
    assert!(short.spent() < exact);
}

#[test]
fn real_cpu44_fallback_and_selector_share_one_allowance_and_rng() {
    let (observation, memory, history) = scene();
    let mut rng = CpuRng::from_seed(memory.key(), [46; 32]);
    let mut work = WorkBudget::new(super::super::WORK_ALLOWANCE);
    let ordinary = super::super::plan_blocked_births_v42_with_origin(
        &observation,
        &memory,
        &history,
        observation.parts().tick,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    let fallback_spent = work.spent();
    let fallback_draws = rng.draws();
    let selected = select_teleporter_recovery(
        &observation,
        &history,
        observation.parts().tick,
        &memory,
        None,
        ordinary,
        &mut work,
    );
    assert!(matches!(selected, TeleporterRecoveryChoice::Step(_)));
    assert_eq!((fallback_spent, work.spent()), (10_926, 21_478));
    assert_eq!(rng.draws(), fallback_draws);
}

#[test]
fn captured_scene_is_supported_with_respawn_lairs_disabled() {
    let (observation, memory, _) = scene();
    let environment = observation.environment().unwrap().clone();
    let pits = observation.spike_pits().cloned();
    let observation =
        Observation::new_rules_v10(observation.into_parts(), environment, None, pits).unwrap();
    let mut history = ObservedHazardMemory::new(memory.key());
    history.observe(&observation).unwrap();
    assert!(matches!(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut WorkBudget::new(65_536),
        ),
        TeleporterRecoveryChoice::Step(_)
    ));
}
