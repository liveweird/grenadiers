use crate::cpu::{Observation, VisibleActor};
use crate::game::{CellId, Direction, Position};

pub(super) fn alignment_direction(
    actor: &VisibleActor,
    cell: CellId,
    threshold: u32,
) -> Option<Direction> {
    let center = Position::center(cell);
    let x_difference = actor.position.x.abs_diff(center.x);
    let y_difference = actor.position.y.abs_diff(center.y);
    let x_violates = x_difference > threshold;
    let y_violates = y_difference > threshold;
    let align_x =
        x_violates && (!y_violates || matches!(actor.heading, Direction::East | Direction::West));
    if !x_violates && !y_violates {
        None
    } else if align_x && actor.position.x > center.x {
        Some(Direction::West)
    } else if align_x && actor.position.x < center.x {
        Some(Direction::East)
    } else if actor.position.y > center.y {
        Some(Direction::North)
    } else if actor.position.y < center.y {
        Some(Direction::South)
    } else {
        None
    }
}

pub(super) fn centered_for_placement(
    observation: &Observation,
    actor: &VisibleActor,
    cell: CellId,
) -> bool {
    let center = Position::center(cell);
    let threshold = placement_center_threshold(observation);
    actor.position.x.abs_diff(center.x) <= threshold
        && actor.position.y.abs_diff(center.y) <= threshold
}

pub(super) fn placement_center_threshold(observation: &Observation) -> u32 {
    observation
        .parts()
        .rules
        .settings
        .turn_assist_units()
        .min(64) as u32
}
