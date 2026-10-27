use crate::cpu::{Observation, VisibleActor};
use crate::game::{CellId, Direction, Position};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum TurnAlignment {
    Ready,
    Correct(Direction),
}

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
    } else if align_x {
        axis_alignment(
            actor.position.x,
            center.x,
            Direction::West,
            Direction::East,
            threshold,
        )
    } else {
        axis_alignment(
            actor.position.y,
            center.y,
            Direction::North,
            Direction::South,
            threshold,
        )
    }
}

pub(super) fn ordinary_turn_alignment(
    actor: &VisibleActor,
    cell: CellId,
    desired: Direction,
    tolerance: u32,
) -> TurnAlignment {
    if same_axis(actor.heading, desired) {
        return TurnAlignment::Ready;
    }
    let center = Position::center(cell);
    let (coordinate, target, negative, positive) = match actor.heading {
        Direction::East | Direction::West => {
            (actor.position.x, center.x, Direction::West, Direction::East)
        }
        Direction::North | Direction::South => (
            actor.position.y,
            center.y,
            Direction::North,
            Direction::South,
        ),
    };
    match axis_alignment(coordinate, target, negative, positive, tolerance) {
        Some(direction) => TurnAlignment::Correct(direction),
        None => TurnAlignment::Ready,
    }
}

fn axis_alignment(
    coordinate: i32,
    target: i32,
    negative: Direction,
    positive: Direction,
    threshold: u32,
) -> Option<Direction> {
    if coordinate.abs_diff(target) <= threshold {
        None
    } else if coordinate > target {
        Some(negative)
    } else {
        Some(positive)
    }
}

fn same_axis(first: Direction, second: Direction) -> bool {
    matches!(
        (first, second),
        (
            Direction::North | Direction::South,
            Direction::North | Direction::South
        ) | (
            Direction::East | Direction::West,
            Direction::East | Direction::West
        )
    )
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
