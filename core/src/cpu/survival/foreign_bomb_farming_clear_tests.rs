use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_ordinary_plant_to_refuge};
use crate::cpu::{BombKnowledge, MobContactLease, MovementIntent, ObservedActor, WorkBudget};
use crate::game::{CellId, Direction, EntityId, Position, StormCloudView, Tick};

use super::foreign_bomb_farming_clear_fixture::{FARM_TARGET, SETTLED_STOP, captured_clear_basis};
use super::{WORK_ALLOWANCE, plant_value::useful_plant_for};
use crate::cpu::policy::StrategicTarget;

#[test]
fn clear_tick_432_preserves_the_actual_public_scene_and_empty_history() {
    let captured = captured_clear_basis();
    let parts = captured.observation.parts();
    assert_eq!(parts.tick, SETTLED_STOP);
    assert_eq!(parts.rules.running_updates, 252);
    assert_eq!(parts.rules.remaining_ticks, 10_548);
    assert_eq!(parts.arena.blocks.len(), 46);
    assert!(!parts.arena.blocks.contains(&CellId::new(1, 8)));
    assert!(!parts.arena.blocks.contains(&CellId::new(13, 8)));
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        unreachable!()
    };
    assert!(bombs.is_empty());
    assert!(parts.flames.is_empty());
    let actors = parts
        .actors
        .iter()
        .map(|actor| match actor {
            ObservedActor::Visible(actor) => (
                actor.id,
                actor.position,
                actor.heading,
                actor.live_bomb_count,
            ),
            ObservedActor::Hidden { .. } => unreachable!(),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actors[1],
        (
            EntityId(2),
            Position {
                x: 11_327,
                y: 1_536
            },
            Direction::East,
            0
        )
    );
    assert_eq!(
        actors[2],
        (
            EntityId(3),
            Position {
                x: 3_479,
                y: 11_776
            },
            Direction::East,
            0
        )
    );
    assert_eq!(
        actors[3],
        (
            EntityId(4),
            Position {
                x: 11_881,
                y: 11_776
            },
            Direction::West,
            0
        )
    );
    assert_eq!(captured.hazard_memory.retained_samples(EntityId(0)), 0);
    assert_eq!(captured.hazard_memory.retained_samples(EntityId(5)), 0);
    assert_eq!(
        captured.observation.environment().unwrap().storm_cloud(),
        Some(StormCloudView {
            position: Position { x: 7_648, y: 6_656 },
            direction: Direction::East,
            leg_remaining_ticks: 4,
        })
    );
}

#[test]
fn hypothesized_row_two_refuge_has_no_raw_plant_route() {
    let captured = captured_clear_basis();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let result = complete_candidate(&captured, CellId::new(13, 2), &mut work);
    assert_eq!(result, Err("plant:Route(NoRoute)".to_owned()));
    assert_eq!(work.spent(), 1_345);
}

#[test]
fn southern_refuge_fits_the_existing_cap_and_needs_its_exact_charged_work() {
    let captured = captured_clear_basis();
    let mut cap = WorkBudget::new(8_192);
    let cap_result = complete_candidate(&captured, CellId::new(13, 3), &mut cap);
    let mut exact = WorkBudget::new(3_889);
    let exact_result = complete_candidate(&captured, CellId::new(13, 3), &mut exact);
    let mut one_short = WorkBudget::new(3_888);
    let short_result = complete_candidate(&captured, CellId::new(13, 3), &mut one_short);
    assert_eq!(cap_result, exact_result);
    assert!(exact_result.is_ok());
    assert!(short_result.unwrap_err().contains("WorkExhausted"));
    assert_eq!(cap.spent(), 3_889);
    assert_eq!(exact.spent(), 3_889);
    // The final eight-unit charge fails atomically, leaving seven units unused.
    assert_eq!(one_short.spent(), 3_881);
    assert_eq!(one_short.remaining(), 7);
}

#[test]
fn explicitly_named_alternate_refuges_are_separate_diagnostics() {
    let captured = captured_clear_basis();
    let mut southern_work = WorkBudget::new(WORK_ALLOWANCE);
    let southern = complete_candidate(&captured, CellId::new(13, 3), &mut southern_work).unwrap();
    assert_eq!(southern.0.placed_at, Tick(439));
    assert_eq!(southern.0.cell, CellId::new(11, 1));
    assert_eq!(southern.0.detonates_at, Tick(619));
    assert_eq!(southern.1, Tick(649));
    assert_eq!(southern.2, Some(true));
    assert_eq!(southern.3, Tick(649));
    assert_eq!(southern_work.spent(), 3_889);

    let mut northern_work = WorkBudget::new(WORK_ALLOWANCE);
    let northern = complete_candidate(&captured, CellId::new(13, 1), &mut northern_work);
    assert_eq!(northern, Err("plant:Route(NoRoute)".to_owned()));
    assert_eq!(northern_work.spent(), 1_208);
}

fn complete_candidate(
    captured: &super::foreign_bomb_farming_clear_fixture::ClearBasis,
    refuge: CellId,
    work: &mut WorkBudget,
) -> Result<(crate::cpu::routing::PlannedBomb, Tick, Option<bool>, Tick), String> {
    let plant = plan_lair_ordinary_plant_to_refuge(
        &captured.observation,
        &captured.hazard_memory,
        PlantRequest {
            execution_tick: Tick(439),
            settled_stop_tick: SETTLED_STOP,
            first_movement: MovementIntent::Direction(Direction::East),
            limits: RouteLimits::default(),
        },
        refuge,
        work,
    )
    .map_err(|error| format!("plant:{error:?}"))?;
    let useful = useful_plant_for(
        &captured.observation,
        plant.placement(),
        StrategicTarget::Block { cell: FARM_TARGET },
        work,
    );
    let contact = MobContactLease::certify_route(&captured.observation, plant.route(), work)
        .map_err(|error| format!("contact:{error:?}"))?;
    Ok((
        *plant.placement(),
        plant.route().survive_until(),
        useful,
        contact.safe_until(),
    ))
}
