use super::movement::MovementOutcome;
use super::{Direction, MovementTrace, Position, SeatId};
use crate::game::MovementSegment;

fn outcome(
    start: Position,
    budget: i32,
    legs: &[(Direction, i32)],
    forward_leg: Option<usize>,
) -> MovementOutcome {
    let mut trace = MovementTrace::new(start);
    let mut cursor = start;
    let mut forward_segment = None;
    for (index, &(direction, distance)) in legs.iter().enumerate() {
        let end = match direction {
            Direction::East => Position {
                x: cursor.x + distance,
                y: cursor.y,
            },
            Direction::West => Position {
                x: cursor.x - distance,
                y: cursor.y,
            },
            Direction::South => Position {
                x: cursor.x,
                y: cursor.y + distance,
            },
            Direction::North => Position {
                x: cursor.x,
                y: cursor.y - distance,
            },
        };
        let segment = MovementSegment::new(cursor, end);
        trace.push(segment);
        if forward_leg == Some(index) {
            forward_segment = Some(segment);
        }
        cursor = end;
    }
    MovementOutcome {
        trace,
        kick: None,
        original_budget: budget,
        forward_segment,
    }
}

fn entry(holder: MovementOutcome, rival: MovementOutcome) -> Option<Direction> {
    super::shove_geometry::contact_entry_direction(SeatId::One, &[holder, rival])
}

#[test]
fn tangent_motion_needs_positive_area_but_one_unit_of_entry_qualifies() {
    let holder = Position { x: 4_000, y: 4_000 };
    let tangent = Position { x: 4_691, y: 4_640 };
    assert_eq!(
        entry(
            outcome(holder, 51, &[(Direction::East, 51)], Some(0)),
            outcome(tangent, 0, &[], None),
        ),
        None
    );

    let one_unit_inside = Position { x: 4_690, y: 4_639 };
    assert_eq!(
        entry(
            outcome(holder, 51, &[(Direction::East, 51)], Some(0)),
            outcome(one_unit_inside, 0, &[], None),
        ),
        Some(Direction::East)
    );
}

#[test]
fn exact_time_geometry_handles_different_budgets_parallel_and_opposing_motion() {
    let holder = Position { x: 4_000, y: 4_000 };
    let rival = Position { x: 4_641, y: 4_000 };
    assert_eq!(
        entry(
            outcome(holder, 86, &[(Direction::East, 86)], Some(0)),
            outcome(rival, 51, &[(Direction::East, 51)], None),
        ),
        Some(Direction::East),
        "the faster parallel path closes the one-unit gap"
    );
    assert_eq!(
        entry(
            outcome(holder, 86, &[(Direction::East, 86)], Some(0)),
            outcome(rival, 51, &[(Direction::West, 51)], None),
        ),
        Some(Direction::East),
        "opposing paths close the same leading face sooner"
    );
}

#[test]
fn correction_then_forward_and_blocked_stationary_remainder_use_actual_timing() {
    let corrected = outcome(
        Position { x: 4_000, y: 4_020 },
        51,
        &[(Direction::North, 20), (Direction::East, 31)],
        Some(1),
    );
    assert_eq!(
        entry(
            corrected,
            outcome(Position { x: 4_650, y: 4_000 }, 0, &[], None)
        ),
        Some(Direction::East)
    );

    let short_forward = outcome(
        Position { x: 4_000, y: 4_000 },
        51,
        &[(Direction::East, 10)],
        Some(0),
    );
    assert_eq!(
        entry(
            short_forward,
            outcome(Position { x: 4_649, y: 4_000 }, 0, &[], None)
        ),
        Some(Direction::East),
        "entry occurs at the forward endpoint before the blocked stationary remainder"
    );

    assert_eq!(
        entry(
            outcome(
                Position { x: 4_000, y: 4_000 },
                51,
                &[(Direction::East, 10)],
                Some(0),
            ),
            outcome(
                Position { x: 4_648, y: 4_655 },
                51,
                &[(Direction::North, 20)],
                None,
            ),
        ),
        None,
        "transverse entry occurs after the short forward leg has stopped; normalizing ten units over the full budget would falsely arm it"
    );
}

#[test]
fn initial_overlap_must_actually_separate_before_reentry() {
    let holder = outcome(
        Position { x: 4_000, y: 4_000 },
        86,
        &[(Direction::North, 51), (Direction::East, 35)],
        Some(1),
    );
    let rival = outcome(
        Position { x: 4_600, y: 4_600 },
        86,
        &[(Direction::East, 51), (Direction::North, 35)],
        None,
    );
    assert_eq!(entry(holder, rival), Some(Direction::East));

    let never_separates = outcome(
        Position { x: 4_000, y: 4_000 },
        86,
        &[(Direction::North, 20), (Direction::East, 66)],
        Some(1),
    );
    assert_eq!(
        entry(
            never_separates,
            outcome(Position { x: 4_600, y: 4_600 }, 0, &[], None)
        ),
        None
    );
}

#[path = "bracelet_collision_proof_tests.rs"]
mod collision_proofs;
