use crate::game::types::UNITS_PER_CELL;
use crate::game::{Direction, Position};

pub(super) fn translated(position: Position, direction: Direction, distance: i32) -> Position {
    match direction {
        Direction::North => Position {
            y: position.y - distance,
            ..position
        },
        Direction::East => Position {
            x: position.x + distance,
            ..position
        },
        Direction::South => Position {
            y: position.y + distance,
            ..position
        },
        Direction::West => Position {
            x: position.x - distance,
            ..position
        },
    }
}

pub(super) fn alignment_coordinate(direction: Direction, position: Position) -> i32 {
    match direction {
        Direction::East | Direction::West => position.y,
        Direction::North | Direction::South => position.x,
    }
}

pub(super) fn nearest_center(value: i32) -> i32 {
    value.div_euclid(UNITS_PER_CELL) * UNITS_PER_CELL + UNITS_PER_CELL / 2
}

pub(super) fn correction_direction(direction: Direction, correction: i32) -> Direction {
    match (direction, correction.is_positive()) {
        (Direction::East | Direction::West, true) => Direction::South,
        (Direction::East | Direction::West, false) => Direction::North,
        (Direction::North | Direction::South, true) => Direction::East,
        (Direction::North | Direction::South, false) => Direction::West,
    }
}

pub(super) fn same_axis(left: Direction, right: Direction) -> bool {
    matches!(
        (left, right),
        (
            Direction::North | Direction::South,
            Direction::North | Direction::South
        ) | (
            Direction::East | Direction::West,
            Direction::East | Direction::West
        )
    )
}

pub(super) const fn opposite(direction: Direction) -> Direction {
    match direction {
        Direction::North => Direction::South,
        Direction::East => Direction::West,
        Direction::South => Direction::North,
        Direction::West => Direction::East,
    }
}

pub(super) const fn direction_bit(direction: Direction) -> u8 {
    match direction {
        Direction::North => 1,
        Direction::East => 2,
        Direction::South => 4,
        Direction::West => 8,
    }
}
