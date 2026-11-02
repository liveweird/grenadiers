use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::{CpuRng, ManualIntent, MovementIntent, WorkBudget};
use crate::game::{CellId, Direction, Tick};

use super::farm_yield_fixture::{
    CPU, EARLY_HIGH_YIELD_ORIGIN, FARM_TARGET, HIGH_YIELD_ORIGIN, SECOND_BLOCK, START,
    collected_basis_at,
};
use super::{ExecutionCertificate, WORK_ALLOWANCE, plan_hard_farm_yield_v59_with_origin};

#[test]
fn hard_v59_promotes_only_the_equal_distance_origin_for_the_same_farm_target() {
    let basis = collected_basis_at(START);
    let run = |seed| {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let plan = plan_hard_farm_yield_v59_with_origin(
            &basis.observation,
            &UtilityMemory::new(basis.key),
            &basis.history,
            basis.observation.parts().tick,
            None,
            &mut CpuRng::from_seed(basis.key, [seed; 32]),
            &mut work,
        )
        .unwrap();
        (plan, work.spent())
    };
    let (planned, work) = run(59);
    let (other_rng, other_work) = run(7);
    assert_eq!(planned.survival, other_rng.survival);
    assert_eq!(planned.contact.is_some(), other_rng.contact.is_some());
    assert_eq!(work, other_work);
    let intent = planned
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Farm);
    assert_eq!(intent.target, StrategicTarget::Block { cell: FARM_TARGET });
    assert_eq!(intent.approach_cell, Some(HIGH_YIELD_ORIGIN));
    assert_eq!(intent.last_distance, 3);
    assert_eq!(
        planned.survival.control.intent.movement,
        MovementIntent::Direction(Direction::South)
    );
    assert_eq!(planned.survival.control.intent.manual, None);
    assert!(planned.survival.execution.is_none());
    assert_eq!(work, 21_370);
}

#[test]
fn hard_v59_south_path_intermediate_has_a_live_two_block_plant() {
    let mut basis = collected_basis_at(EARLY_HIGH_YIELD_ORIGIN);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_farm_yield_v59_with_origin(
        &basis.observation,
        &UtilityMemory::new(basis.key),
        &basis.history,
        basis.observation.parts().tick,
        None,
        &mut CpuRng::from_seed(basis.key, [59; 32]),
        &mut work,
    )
    .unwrap();
    let intent = planned
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    assert_eq!(intent.target, StrategicTarget::Block { cell: FARM_TARGET });
    assert_eq!(intent.approach_cell, Some(HIGH_YIELD_ORIGIN));
    assert_eq!(
        planned.survival.control.intent.movement,
        MovementIntent::Direction(Direction::South)
    );
    assert_eq!(
        planned.survival.control.intent.manual,
        Some(ManualIntent::Bomb)
    );
    assert_eq!(work.spent(), 36_436);
    let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
        panic!("v59 lost the complete early high-yield plant")
    };
    assert_eq!(certificate.placement().cell, EARLY_HIGH_YIELD_ORIGIN);
    assert_eq!(certificate.placement().placed_at, Tick(49));
    assert_eq!(certificate.placement().detonates_at, Tick(229));
    assert_eq!(certificate.route().survive_until(), Tick(259));
    super::farm_yield_witness_tests::execute_through_flames(
        &mut basis,
        *certificate,
        MovementIntent::Direction(Direction::South),
    );
    let blocks = basis.state.world().arena().blocks();
    assert!(!blocks.contains(&FARM_TARGET));
    assert!(!blocks.contains(&SECOND_BLOCK));
    assert_eq!(blocks.len(), 4);
    assert!(basis.state.actor(CPU).unwrap().alive());
}

#[test]
fn archived_hard_v58_baseline_remains_the_nearest_origin() {
    let basis = collected_basis_at(START);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = super::plan_hard_exit_pressure_v58_with_origin(
        &basis.observation,
        &UtilityMemory::new(basis.key),
        &basis.history,
        basis.observation.parts().tick,
        None,
        &mut CpuRng::from_seed(basis.key, [58; 32]),
        &mut work,
    )
    .unwrap();
    assert_eq!(
        planned
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap()
            .approach_cell,
        Some(CellId::new(3, 6))
    );
    assert_eq!(work.spent(), 21_360);
}
