use super::tests::{assert_fallback, fallback, scene, unrelated_origin};
use super::*;
use crate::cpu::forecast::{ForecastLimits, forecast};
use crate::cpu::routing::TeleporterStepPhase;
use crate::cpu::{CpuRng, MemoryKey, MovementIntent, ObservedActor};
use crate::game::{CellId, Direction, PickupKind, Position, Tick};

const ASSUMED_SETTLED_STOP: Tick = Tick(1_381);

#[test]
fn assumed_hard51_worker_path_selects_the_captured_south_approach_with_shared_work() {
    use crate::cpu::routing::teleporter_step_v50_fixture::{CPU, TARGET, captured_tick_1381};

    let observation = captured_tick_1381();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let memory = UtilityMemory::new(key);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    // The capture contains no private controller state. This fixed seed and
    // the settled-stop clock are explicit worker-characterization premises.
    let mut rng = CpuRng::from_seed(key, [51; 32]);
    let mut work = WorkBudget::new(super::super::WORK_ALLOWANCE);
    let ordinary = super::super::plan_hard_blocked_births_v43_with_origin(
        &observation,
        &memory,
        &history,
        ASSUMED_SETTLED_STOP,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    let ordinary_spent = work.spent();
    let ordinary_draws = rng.draws();
    let guidance = crate::cpu::policy::teleporter_guidance_v50(
        &observation,
        &memory,
        &mut WorkBudget::new(32_768),
    )
    .unwrap();
    let selected = select_teleporter_recovery_v50(
        &observation,
        &history,
        ASSUMED_SETTLED_STOP,
        &memory,
        None,
        ordinary,
        &mut work,
    );
    let TeleporterRecoveryChoice::Step(plan) = selected else {
        panic!("the captured Hard51-equivalent worker path should select a step")
    };
    assert_eq!(plan.step.target(), TARGET);
    assert_eq!(plan.step.phase(), TeleporterStepPhase::Approach);
    assert_eq!(
        plan.step.movement(),
        MovementIntent::Direction(Direction::South)
    );
    let waypoint = guidance
        .candidates
        .iter()
        .flatten()
        .find(|candidate| candidate.movement == plan.step.movement())
        .unwrap()
        .waypoint;
    assert_eq!(waypoint, CellId::new(9, 10));
    assert!(
        plan.step
            .nominal_makes_progress_toward(waypoint, &mut WorkBudget::new(65_536))
            .unwrap()
    );
    assert_eq!((ordinary_spent, work.spent()), (12_871, 26_168));
    assert_eq!(rng.draws(), ordinary_draws);
}

#[test]
fn four_player_collect_reports_the_frozen_candidate_allowance_boundary() {
    use crate::cpu::routing::teleporter_step_v50_fixture::{CPU, TARGET, captured_tick_1381};

    let observation = captured_tick_1381()
        .map_parts(|parts| {
            let ObservedActor::Visible(actor) = &mut parts.actors[CPU.index()] else {
                unreachable!()
            };
            actor.position = Position {
                x: 13_824,
                y: 7_140,
            };
            actor.heading = Direction::South;
            let ObservedActor::Visible(partner) = &mut parts.actors[3] else {
                unreachable!()
            };
            partner.position = Position::center(CellId::new(13, 11));
        })
        .unwrap();
    let key = MemoryKey {
        world: observation.parts().world,
        cpu_seat: CPU,
    };
    let memory = UtilityMemory::new(key);
    let mut history = ObservedHazardMemory::new(key);
    history.observe(&observation).unwrap();
    let mut setup_work = WorkBudget::new(32_768);
    let guidance =
        crate::cpu::policy::teleporter_guidance_v50(&observation, &memory, &mut setup_work)
            .unwrap();
    let _hazards = forecast(
        &observation,
        &history,
        None,
        ForecastLimits::default(),
        &mut setup_work,
    )
    .unwrap();
    let candidate_count = guidance.candidates.iter().flatten().count();
    let frozen = candidate_allowances(setup_work.remaining(), candidate_count);
    assert_eq!((candidate_count, frozen[0]), (2, 16_212));
    assert!(frozen[0] >= 15_156, "first slot covers the measured proof");

    let mut work = WorkBudget::new(65_536);
    let selected = select_teleporter_recovery_v50(
        &observation,
        &history,
        ASSUMED_SETTLED_STOP,
        &memory,
        None,
        fallback(memory),
        &mut work,
    );
    let TeleporterRecoveryChoice::Step(plan) = selected else {
        panic!("the frozen first candidate has enough work for Collect")
    };
    assert_eq!(plan.step.target(), TARGET);
    assert_eq!(plan.step.phase(), TeleporterStepPhase::Collect);
    assert_eq!(
        plan.step.movement(),
        MovementIntent::Direction(Direction::South)
    );
    assert_eq!(work.spent(), 15_506);
}

#[test]
fn legacy_selector_rejects_two_teleporters() {
    let (basis, memory, _) = scene();
    let observation = basis
        .map_parts(|parts| {
            parts
                .pickups
                .iter_mut()
                .find(|pickup| pickup.cell == CellId::new(13, 7))
                .unwrap()
                .kind = PickupKind::Teleporter;
        })
        .unwrap();
    let mut history = ObservedHazardMemory::new(memory.key());
    history.observe(&observation).unwrap();
    assert_fallback(
        select_teleporter_recovery(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut WorkBudget::new(65_536),
        ),
        memory,
    );
}

#[test]
fn one_teleporter_preserves_legacy_step_and_work() {
    let (observation, memory, history) = scene();
    let mut legacy_work = WorkBudget::new(65_536);
    let legacy = select_teleporter_recovery(
        &observation,
        &history,
        observation.parts().tick,
        &memory,
        None,
        fallback(memory),
        &mut legacy_work,
    );
    let mut v50_work = WorkBudget::new(65_536);
    let v50 = select_teleporter_recovery_v50(
        &observation,
        &history,
        observation.parts().tick,
        &memory,
        None,
        fallback(memory),
        &mut v50_work,
    );
    let TeleporterRecoveryChoice::Step(legacy) = legacy else {
        panic!("legacy selector should certify the captured scene")
    };
    let TeleporterRecoveryChoice::Step(v50) = v50 else {
        panic!("v50 selector should preserve the one-Teleporter result")
    };
    assert_eq!(v50.step.target(), legacy.step.target());
    assert_eq!(v50.step.movement(), legacy.step.movement());
    assert_eq!(v50.step.phase(), legacy.step.phase());
    assert_eq!(v50.next_memory, legacy.next_memory);
    assert_eq!(v50_work.spent(), legacy_work.spent());
}

#[test]
fn origin_and_zero_allowance_return_the_exact_fallback_without_work() {
    let (observation, memory, history) = scene();
    let mut origin_work = WorkBudget::new(65_536);
    assert_fallback(
        select_teleporter_recovery_v50(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            Some(&unrelated_origin()),
            fallback(memory),
            &mut origin_work,
        ),
        memory,
    );
    assert_eq!(origin_work.spent(), 0);

    let mut zero_work = WorkBudget::new(0);
    assert_fallback(
        select_teleporter_recovery_v50(
            &observation,
            &history,
            observation.parts().tick,
            &memory,
            None,
            fallback(memory),
            &mut zero_work,
        ),
        memory,
    );
    assert_eq!(zero_work.spent(), 0);
}

#[test]
fn one_short_of_the_measured_allowance_preserves_the_exact_fallback() {
    let (observation, memory, history) = scene();
    let mut measured = WorkBudget::new(65_536);
    assert!(matches!(
        select_teleporter_recovery_v50(
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
    let mut short = WorkBudget::new(exact - 1);
    assert_fallback(
        select_teleporter_recovery_v50(
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
    assert!(short.spent() < exact);
}
