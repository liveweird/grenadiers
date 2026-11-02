use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::{CpuRng, ManualIntent, MovementIntent, WorkBudget};
use crate::game::{BlastKind, CellId, Direction, EnabledItems, Position, Tick};

use super::exit_pressure_fixture::{
    CPU, NORTH_ORIGIN, RIVAL, SOUTH_EXIT_BLOCK, START, WEST_ORIGIN, collected_basis_at,
    collected_basis_at_with_lairs,
};
use super::{
    ExecutionCertificate, ObservedBlastGeometry, WORK_ALLOWANCE,
    plan_hard_corner_refuges_v57_with_origin, plan_hard_exit_pressure_v58_with_origin,
};

#[test]
fn hard_v57_naturally_collects_range_three_then_prefers_the_north_origin_tie() {
    let basis = collected_basis_at(START);
    let actor = basis
        .observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor) if actor.seat == CPU => Some(actor),
            _ => None,
        })
        .unwrap();
    assert_eq!(actor.bomb_range, 3);
    assert!(basis.observation.parts().pickups.is_empty());
    assert_eq!(basis.observation.parts().arena.blocks, [SOUTH_EXIT_BLOCK]);
    assert_eq!(
        basis.observation.parts().rules.settings.enabled_items(),
        EnabledItems::ALL
    );
    assert!(basis.observation.respawn_lairs().is_none());
    assert!(basis.observation.spike_pits().is_none());
    assert_eq!(basis.observation.environment().unwrap().storm_cloud(), None);
    assert_eq!(
        START.column.abs_diff(NORTH_ORIGIN.column) + START.row.abs_diff(NORTH_ORIGIN.row),
        2
    );
    assert_eq!(
        START.column.abs_diff(WEST_ORIGIN.column) + START.row.abs_diff(WEST_ORIGIN.row),
        2
    );

    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_corner_refuges_v57_with_origin(
        &basis.observation,
        &UtilityMemory::new(basis.key),
        &basis.history,
        basis.observation.parts().tick,
        None,
        &mut CpuRng::from_seed(basis.key, [57; 32]),
        &mut work,
    )
    .unwrap();
    let intent = planned
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(intent.target, StrategicTarget::Opponent { seat: RIVAL });
    assert_eq!(intent.approach_cell, Some(NORTH_ORIGIN));
    assert_eq!(
        planned.survival.control.intent.movement,
        MovementIntent::Direction(Direction::East)
    );
    assert!(planned.survival.execution.is_none());
    assert_eq!(work.spent(), 21_203);
}

#[test]
fn hard_v58_promotes_the_equal_distance_west_origin_with_two_open_exit_branches() {
    let basis = collected_basis_at(START);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_exit_pressure_v58_with_origin(
        &basis.observation,
        &UtilityMemory::new(basis.key),
        &basis.history,
        basis.observation.parts().tick,
        None,
        &mut CpuRng::from_seed(basis.key, [58; 32]),
        &mut work,
    )
    .unwrap();
    let intent = planned
        .survival
        .control
        .next_memory
        .strategic_intent()
        .unwrap();
    assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
    assert_eq!(intent.target, StrategicTarget::Opponent { seat: RIVAL });
    assert_eq!(intent.approach_cell, Some(WEST_ORIGIN));
    assert_eq!(
        planned.survival.control.intent.movement,
        MovementIntent::Direction(Direction::South)
    );
    assert!(planned.survival.execution.is_none());
    assert!(work.spent() <= WORK_ALLOWANCE);
}

#[test]
fn hard_v58_west_origin_keeps_the_complete_plant_and_escape_authority() {
    let basis = collected_basis_at(WEST_ORIGIN);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_exit_pressure_v58_with_origin(
        &basis.observation,
        &UtilityMemory::new(basis.key),
        &basis.history,
        basis.observation.parts().tick,
        None,
        &mut CpuRng::from_seed(basis.key, [58; 32]),
        &mut work,
    )
    .unwrap();
    let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
        panic!("west origin lost its complete plant proof")
    };
    assert_eq!(certificate.placement().cell, WEST_ORIGIN);
    assert_eq!(certificate.placement().placed_at, Tick(49));
    assert_eq!(certificate.placement().detonates_at, Tick(229));
    assert_eq!(certificate.route().survive_until(), Tick(259));
    assert_eq!(
        super::plant_value::useful_plant_for(
            &basis.observation,
            certificate.placement(),
            StrategicTarget::Opponent { seat: RIVAL },
            &mut work,
        ),
        Some(true)
    );
    assert!(work.spent() <= WORK_ALLOWANCE);
}

#[test]
fn hard_v58_lair_planner_uses_the_farm_exit_pressure_variant() {
    let basis = collected_basis_at_with_lairs(START, true);
    assert!(basis.observation.respawn_lairs().is_some());
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_exit_pressure_v58_with_origin(
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
        Some(WEST_ORIGIN)
    );
    assert!(work.spent() <= WORK_ALLOWANCE);
}

#[test]
fn both_equal_distance_origins_have_complete_useful_escape_proofs_in_closed_contact_model() {
    for (origin, expected_work) in [(NORTH_ORIGIN, 3_981), (WEST_ORIGIN, 3_977)] {
        let basis = collected_basis_at(origin);
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let planned = plan_hard_corner_refuges_v57_with_origin(
            &basis.observation,
            &UtilityMemory::new(basis.key),
            &basis.history,
            basis.observation.parts().tick,
            None,
            &mut CpuRng::from_seed(basis.key, [57; 32]),
            &mut work,
        )
        .unwrap();
        let intent = planned
            .survival
            .control
            .next_memory
            .strategic_intent()
            .unwrap();
        assert_eq!(intent.objective, StrategicObjectiveKind::Attack);
        assert_eq!(intent.target, StrategicTarget::Opponent { seat: RIVAL });
        assert_eq!(intent.approach_cell, Some(origin));
        assert_eq!(
            planned.survival.control.intent.manual,
            Some(ManualIntent::Bomb)
        );
        let has_contact = planned.contact.is_some();
        let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
            panic!("origin {origin:?} did not receive a complete plant proof")
        };
        assert_eq!(certificate.placement().cell, origin);
        assert_eq!(certificate.placement().placed_at, Tick(49));
        assert_eq!(certificate.placement().detonates_at, Tick(229));
        assert_eq!(certificate.route().survive_until(), Tick(259));
        assert_eq!(
            super::plant_value::useful_plant_for(
                &basis.observation,
                certificate.placement(),
                StrategicTarget::Opponent { seat: RIVAL },
                &mut work,
            ),
            Some(true)
        );
        assert!(basis.observation.respawn_lairs().is_none());
        assert!(!has_contact);
        assert_eq!(
            planned.survival.control.intent.movement,
            MovementIntent::Stop
        );
        assert_eq!(work.spent(), expected_work);
    }
}

#[test]
fn west_origin_covers_both_open_lateral_exits_while_north_covers_only_north() {
    let basis = collected_basis_at(START);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let geometry = ObservedBlastGeometry::new(&basis.observation, &mut work).unwrap();
    let north = Position::center(CellId::new(7, 6));
    let east = Position::center(CellId::new(8, 7));
    let west = Position::center(CellId::new(6, 7));
    assert_eq!(
        geometry.threatens_position(NORTH_ORIGIN, 3, BlastKind::Standard, north, &mut work),
        Some(true)
    );
    assert_eq!(
        geometry.threatens_position(NORTH_ORIGIN, 3, BlastKind::Standard, east, &mut work),
        Some(false)
    );
    assert_eq!(
        geometry.threatens_position(NORTH_ORIGIN, 3, BlastKind::Standard, west, &mut work),
        Some(false)
    );
    for exit in [east, west] {
        assert_eq!(
            geometry.threatens_position(WEST_ORIGIN, 3, BlastKind::Standard, exit, &mut work),
            Some(true)
        );
    }
}
