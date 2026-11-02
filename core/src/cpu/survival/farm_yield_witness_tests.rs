use crate::cpu::policy::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};
use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant};
use crate::cpu::{BombKnowledge, CpuRng, MovementIntent, WorkBudget};
use crate::game::{Direction, EnabledItems, InputAction, Position, SeatInput, Tick};

use super::farm_yield_fixture::{
    BASELINE_ORIGIN, CPU, FARM_TARGET, FarmYieldBasis, HIGH_YIELD_ORIGIN, SECOND_BLOCK, START,
    collected_basis_at,
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
        execute_through_flames(&mut basis, certificate);
        let blocks = basis.state.world().arena().blocks();
        assert!(!blocks.contains(&FARM_TARGET));
        assert_eq!(blocks.contains(&SECOND_BLOCK), expected_destroyed == 1);
        assert_eq!(6 - blocks.len(), expected_destroyed);
        assert!(basis.state.actor(CPU).unwrap().alive());
    }
}

fn execute_through_flames(
    basis: &mut FarmYieldBasis,
    certificate: crate::cpu::routing::PlantCertificate,
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
        MovementIntent::Stop,
    )
    .unwrap();
    basis
        .state
        .step(&[
            SeatInput {
                seat: CPU,
                action: InputAction::Clear,
            },
            SeatInput {
                seat: CPU,
                action: InputAction::PlaceBomb,
            },
        ])
        .unwrap();
    let mut settled_stop = Some(basis.state.tick());
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
