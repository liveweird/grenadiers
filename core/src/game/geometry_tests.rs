use super::{MovementSegment, MovementTrace};
use crate::game::types::{CellId, HALF_FOOTPRINT, Position, UNITS_PER_CELL};

#[test]
fn swept_overlap_requires_positive_area() {
    let cell = CellId::new(2, 2);
    let touching = Position {
        x: 2 * UNITS_PER_CELL - HALF_FOOTPRINT,
        y: 2 * UNITS_PER_CELL + UNITS_PER_CELL / 2,
    };
    assert!(!MovementSegment::new(touching, touching).overlaps_cell(cell));

    let entering = Position {
        x: touching.x + 1,
        y: touching.y,
    };
    assert!(MovementSegment::new(touching, entering).overlaps_cell(cell));
}

#[test]
fn stationary_trace_keeps_the_starting_footprint() {
    let start = Position::center(CellId::new(4, 4));
    let trace = MovementTrace::new(start);

    assert!(trace.overlaps_cell(CellId::new(4, 4)));
    assert!(!trace.overlaps_cell(CellId::new(5, 4)));
}

#[test]
fn trace_checks_each_piece_instead_of_a_diagonal_chord() {
    let start = Position { x: 512, y: 512 };
    let corner = Position { x: 512, y: 1536 };
    let end = Position { x: 1536, y: 1536 };
    let mut trace = MovementTrace::new(start);
    trace.push(MovementSegment::new(start, corner));
    trace.push(MovementSegment::new(corner, end));

    assert_eq!(trace.len(), 2);
    assert!(!trace.overlaps_cell(CellId::new(1, 0)));
    assert!(trace.overlaps_cell(CellId::new(0, 1)));
    assert!(trace.overlaps_cell(CellId::new(1, 1)));
}
