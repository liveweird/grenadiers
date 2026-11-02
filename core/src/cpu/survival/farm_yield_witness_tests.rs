use crate::cpu::policy::{
    GoalModes, PressureMode, StrategicObjectiveKind, StrategicTarget, UtilityMemory,
};
use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant};
use crate::cpu::{BombKnowledge, CpuRng, ManualIntent, MovementIntent, WorkBudget};
use crate::game::{Direction, EnabledItems, InputAction, Position, SeatInput, Tick};

use super::farm_yield_fixture::{
    BASELINE_ORIGIN, CPU, EARLY_HIGH_YIELD_ORIGIN, FARM_TARGET, FarmYieldBasis, HIGH_YIELD_ORIGIN,
    SECOND_BLOCK, START, collected_basis_at,
};
use super::{
    CommittedRoute, ExecutionCertificate, RouteStatus, WORK_ALLOWANCE,
    plan_hard_exit_pressure_v58_with_origin,
};

#[test]
fn hard_v58_freezes_the_natural_range_three_farm_choice() {
    let basis = collected_basis_at(START);
    let parts = basis.observation.parts();
    let cpu = parts
        .actors
        .iter()
        .find_map(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor) if actor.seat == CPU => Some(actor),
            _ => None,
        })
        .unwrap();
    assert_eq!(parts.tick, Tick(1));
    assert_eq!(cpu.position, Position::center(START));
    assert_eq!(cpu.bomb_range, 3);
    assert_eq!(parts.arena.blocks.len(), 6);
    for cell in [
        FARM_TARGET,
        SECOND_BLOCK,
        crate::game::CellId::new(10, 9),
        crate::game::CellId::new(12, 9),
        crate::game::CellId::new(11, 8),
        crate::game::CellId::new(11, 10),
    ] {
        assert!(parts.arena.blocks.contains(&cell));
    }
    assert_eq!(
        START.column.abs_diff(BASELINE_ORIGIN.column) + START.row.abs_diff(BASELINE_ORIGIN.row),
        3
    );
    assert_eq!(
        START.column.abs_diff(HIGH_YIELD_ORIGIN.column) + START.row.abs_diff(HIGH_YIELD_ORIGIN.row),
        3
    );
    assert!(parts.pickups.is_empty());
    assert!(matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(parts.flames.is_empty());
    assert!(parts.oil.is_empty());
    assert_eq!(parts.rules.settings.enabled_items(), EnabledItems::ALL);
    assert!(basis.observation.respawn_lairs().is_none());
    assert!(basis.observation.spike_pits().is_none());
    assert_eq!(basis.observation.environment().unwrap().storm_cloud(), None);

    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_exit_pressure_v58_with_origin(
        &basis.observation,
        &UtilityMemory::new(basis.key),
        &basis.history,
        parts.tick,
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
    assert_eq!(intent.objective, StrategicObjectiveKind::Farm);
    assert_eq!(intent.target, StrategicTarget::Block { cell: FARM_TARGET });
    assert_eq!(intent.approach_cell, Some(BASELINE_ORIGIN));
    assert_eq!(
        planned.survival.control.intent.movement,
        MovementIntent::Direction(Direction::West)
    );
    assert_eq!(planned.survival.control.intent.manual, None);
    assert!(planned.survival.execution.is_none());
    assert_eq!(work.spent(), 21_360);
}

#[test]
fn both_equal_distance_origins_have_useful_complete_escape_proofs_and_physical_yields() {
    for (origin, expected_destroyed, expected_work) in
        [(BASELINE_ORIGIN, 1, 3_002), (HIGH_YIELD_ORIGIN, 2, 21_854)]
    {
        let mut basis = collected_basis_at(origin);
        assert_eq!(basis.observation.parts().tick, Tick(62));
        let execution_tick = Tick(basis.observation.parts().tick.0 + 7);
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let certificate = plan_lair_ordinary_plant(
            &basis.observation,
            &basis.history,
            PlantRequest {
                execution_tick,
                settled_stop_tick: basis.observation.parts().tick,
                first_movement: MovementIntent::Stop,
                limits: RouteLimits::default(),
            },
            &mut work,
        )
        .unwrap();
        assert_eq!(certificate.placement().cell, origin);
        assert_eq!(certificate.placement().placed_at, Tick(69));
        assert_eq!(certificate.placement().detonates_at, Tick(249));
        assert_eq!(certificate.route().survive_until(), Tick(279));
        assert_eq!(
            super::plant_value::useful_plant_for(
                &basis.observation,
                certificate.placement(),
                StrategicTarget::Block { cell: FARM_TARGET },
                &mut work,
            ),
            Some(true)
        );
        assert_eq!(work.spent(), expected_work);
        execute_through_flames(&mut basis, certificate, MovementIntent::Stop);
        let blocks = basis.state.world().arena().blocks();
        assert!(!blocks.contains(&FARM_TARGET));
        assert_eq!(blocks.contains(&SECOND_BLOCK), expected_destroyed == 1);
        assert_eq!(6 - blocks.len(), expected_destroyed);
        assert!(basis.state.actor(CPU).unwrap().alive());
    }
}

#[test]
fn hard_v58_live_wrapper_plants_only_at_the_current_baseline_origin() {
    for (origin, expected_work) in [(BASELINE_ORIGIN, 3_764), (HIGH_YIELD_ORIGIN, 39_979)] {
        let basis = collected_basis_at(origin);
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
        assert_eq!(intent.objective, StrategicObjectiveKind::Farm);
        assert_eq!(intent.target, StrategicTarget::Block { cell: FARM_TARGET });
        assert_eq!(intent.approach_cell, Some(origin));
        assert_eq!(work.spent(), expected_work);
        if origin == BASELINE_ORIGIN {
            assert_eq!(
                planned.survival.control.intent.movement,
                MovementIntent::Stop
            );
            assert_eq!(
                planned.survival.control.intent.manual,
                Some(ManualIntent::Bomb)
            );
            let Some(ExecutionCertificate::Plant(certificate)) = planned.survival.execution else {
                panic!("live wrapper lost the baseline plant")
            };
            assert_eq!(certificate.placement().cell, BASELINE_ORIGIN);
        } else {
            assert_eq!(
                planned.survival.control.intent.movement,
                MovementIntent::Direction(Direction::North)
            );
            assert_eq!(planned.survival.control.intent.manual, None);
            assert!(planned.survival.execution.is_none());
        }
    }
}

#[test]
fn hard_v58_live_wrapper_at_the_high_yield_south_path_intermediate() {
    let mut basis = collected_basis_at(EARLY_HIGH_YIELD_ORIGIN);
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
    assert_eq!(basis.observation.parts().tick, Tick(42));
    assert_eq!(intent.objective, StrategicObjectiveKind::Farm);
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
        panic!("south-path intermediate lost its live plant")
    };
    assert_eq!(certificate.placement().cell, EARLY_HIGH_YIELD_ORIGIN);
    assert_eq!(certificate.placement().placed_at, Tick(49));
    assert_eq!(certificate.placement().detonates_at, Tick(229));
    assert_eq!(certificate.route().survive_until(), Tick(259));
    execute_through_flames(
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
fn high_yield_live_candidate_is_refused_by_the_bounded_no_lair_plant_tactic() {
    let basis = collected_basis_at(HIGH_YIELD_ORIGIN);
    let memory = UtilityMemory::new(basis.key);
    let mut rng = CpuRng::from_seed(basis.key, [58; 32]);
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let ordinary = super::ordinary_planner::plan(
        &basis.observation,
        &memory,
        &mut rng,
        &mut work,
        true,
        true,
        true,
        true,
        GoalModes::nearest(PressureMode::ExitCoverage),
        false,
        true,
    )
    .unwrap();
    let ordinary_work = work.spent();
    assert_eq!(ordinary.intent.movement, MovementIntent::Stop);
    assert_eq!(ordinary.intent.manual, None);
    assert!(super::strategic_recovery::attained_firing_position(
        &basis.observation,
        &ordinary,
    ));
    let clear = super::deadline::ordinary_movement_is_clear(
        &basis.observation,
        ordinary.intent.movement != MovementIntent::Stop,
        &mut work,
    );
    let before_tactic = work.spent();
    let planned = super::plant_tactic::certified_plant_for_policy(
        &basis.observation,
        &basis.history,
        basis.observation.parts().tick,
        &memory,
        &ordinary,
        true,
        true,
        true,
        super::plant_tactic::PlantSupport::for_policy(true, true),
        &mut work,
    );
    assert_eq!(ordinary_work, 754);
    assert!(!clear);
    assert_eq!(before_tactic, 754);
    let tactic_allowance = (WORK_ALLOWANCE - before_tactic) / 2;
    assert_eq!(tactic_allowance, 32_391);
    let first_candidate_slice = tactic_allowance - (tactic_allowance / 10) * 4;
    assert_eq!(first_candidate_slice, 19_435);
    assert!(21_854 > first_candidate_slice);
    assert!(planned.is_none());
    assert_eq!(work.spent(), 33_089);
}

pub(super) fn execute_through_flames(
    basis: &mut FarmYieldBasis,
    certificate: crate::cpu::routing::PlantCertificate,
    accepted_movement: MovementIntent,
) {
    let placement = *certificate.placement();
    let survive_until = certificate.route().survive_until();
    let admission_tick = Tick(placement.placed_at.0 - 1);
    while basis.state.tick() < admission_tick {
        basis.state.step(&[]).unwrap();
    }
    let current = super::reliable_v27_oil_tests::observe(&basis.state, basis.settings);
    let mut route = CommittedRoute::admit_execution(
        ExecutionCertificate::Plant(Box::new(certificate)),
        basis.observation.clone(),
        &current,
        admission_tick,
        accepted_movement,
    )
    .unwrap();
    let mut inputs = movement_input(accepted_movement);
    inputs.push(SeatInput {
        seat: CPU,
        action: InputAction::PlaceBomb,
    });
    basis.state.step(&inputs).unwrap();
    let mut settled_stop =
        (accepted_movement == MovementIntent::Stop).then_some(basis.state.tick());
    while basis.state.tick() <= survive_until {
        let current = super::reliable_v27_oil_tests::observe(&basis.state, basis.settings);
        assert_ne!(
            route.observe_with_stop(&current, settled_stop),
            RouteStatus::Invalidated
        );
        let status = route.poll_command(basis.state.tick(), settled_stop);
        let inputs = match status {
            RouteStatus::Submit(command) => {
                assert!(route.commit_command(command));
                settled_stop = (command.movement() == MovementIntent::Stop)
                    .then_some(Tick(basis.state.tick().0 + 1));
                movement_input(command.movement())
            }
            RouteStatus::Invalidated => panic!("certified farm escape invalidated"),
            RouteStatus::Completed | RouteStatus::Continue | RouteStatus::Wait => Vec::new(),
        };
        basis.state.step(&inputs).unwrap();
    }
    let after = super::reliable_v27_oil_tests::observe(&basis.state, basis.settings);
    assert!(matches!(&after.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()));
    assert!(after.parts().flames.is_empty());
}

fn movement_input(movement: MovementIntent) -> Vec<SeatInput> {
    let action = match movement {
        MovementIntent::Direction(direction) => InputAction::Press(direction),
        MovementIntent::Stop => InputAction::Clear,
    };
    vec![SeatInput { seat: CPU, action }]
}
