use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::{MovementIntent, Observation, ObservedActor, WorkBudget};
use crate::game::types::UNITS_PER_CELL;
use crate::game::{CellId, Direction, EnabledItems, Equipment, Position};

use super::plant_tests::request;
use super::tests::{key, observation};
use super::{PlantFailure, plan_ordinary_plant};

fn exiting_block() -> Observation {
    let mut parts = observation(Vec::new(), Vec::new()).into_parts();
    parts.rules.settings = parts
        .rules
        .settings
        .with_enabled_items(EnabledItems::WALL_PASS);
    parts.arena.blocks.push(CellId::new(3, 1));
    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
        unreachable!()
    };
    actor.position = Position {
        x: 4 * UNITS_PER_CELL - 16,
        y: Position::center(CellId::new(3, 1)).y,
    };
    actor.heading = Direction::East;
    actor.equipment = Some(Equipment::WallPass);
    Observation::new(parts).unwrap()
}

#[test]
fn first_move_can_exit_a_block_but_plant_origin_must_be_free() {
    let basis = exiting_block();
    let memory = ObservedHazardMemory::new(key());
    let certificate = plan_ordinary_plant(
        &basis,
        &memory,
        request(MovementIntent::Direction(Direction::East)),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    assert_eq!(certificate.placement().cell, CellId::new(4, 1));

    assert_eq!(
        plan_ordinary_plant(
            &basis,
            &memory,
            request(MovementIntent::Stop),
            &mut WorkBudget::new(65_536),
        ),
        Err(PlantFailure::Unsupported)
    );
}
