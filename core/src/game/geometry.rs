use super::Direction;
use super::types::{CellId, HALF_FOOTPRINT, Position, UNITS_PER_CELL};

/// One axis-aligned piece of an actor's movement during a tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovementSegment {
    start: Position,
    end: Position,
}

impl MovementSegment {
    #[must_use]
    pub const fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub const fn start(self) -> Position {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Position {
        self.end
    }

    /// Whether the swept actor footprint has positive-area overlap with a cell.
    #[must_use]
    pub fn overlaps_cell(self, cell: CellId) -> bool {
        let cell_left = i64::from(cell.column) * i64::from(UNITS_PER_CELL);
        let cell_top = i64::from(cell.row) * i64::from(UNITS_PER_CELL);
        let cell_right = cell_left + i64::from(UNITS_PER_CELL);
        let cell_bottom = cell_top + i64::from(UNITS_PER_CELL);
        let swept_left = i64::from(self.start.x.min(self.end.x)) - i64::from(HALF_FOOTPRINT);
        let swept_top = i64::from(self.start.y.min(self.end.y)) - i64::from(HALF_FOOTPRINT);
        let swept_right = i64::from(self.start.x.max(self.end.x)) + i64::from(HALF_FOOTPRINT);
        let swept_bottom = i64::from(self.start.y.max(self.end.y)) + i64::from(HALF_FOOTPRINT);

        swept_left < cell_right
            && swept_right > cell_left
            && swept_top < cell_bottom
            && swept_bottom > cell_top
    }
}

/// Whether an actor footprint at `position` has positive-area overlap with a cell.
#[must_use]
pub fn position_overlaps_cell(position: Position, cell: CellId) -> bool {
    MovementSegment::new(position, position).overlaps_cell(cell)
}

/// Maximum movement permitted by the rectangular arena boundary.
#[must_use]
pub fn boundary_distance(
    position: Position,
    direction: Direction,
    requested: i32,
    width: u16,
    height: u16,
) -> i32 {
    let requested = requested.max(0);
    let right = i32::from(width).saturating_mul(UNITS_PER_CELL);
    let bottom = i32::from(height).saturating_mul(UNITS_PER_CELL);
    match direction {
        Direction::East => requested.min(right - HALF_FOOTPRINT - position.x),
        Direction::West => requested.min(position.x - HALF_FOOTPRINT),
        Direction::South => requested.min(bottom - HALF_FOOTPRINT - position.y),
        Direction::North => requested.min(position.y - HALF_FOOTPRINT),
    }
    .max(0)
}

/// Distance to the near edge of a blocking cell on the requested axis.
///
/// Returns `None` when the footprint does not overlap the cell on the
/// transverse axis or the cell is not ahead of the actor.
#[must_use]
pub fn distance_to_blocking_cell(
    position: Position,
    direction: Direction,
    cell: CellId,
) -> Option<i32> {
    let left = i32::from(cell.column) * UNITS_PER_CELL;
    let top = i32::from(cell.row) * UNITS_PER_CELL;
    let right = left + UNITS_PER_CELL;
    let bottom = top + UNITS_PER_CELL;
    let transverse_overlap = match direction {
        Direction::East | Direction::West => {
            position.y - HALF_FOOTPRINT < bottom && position.y + HALF_FOOTPRINT > top
        }
        Direction::North | Direction::South => {
            position.x - HALF_FOOTPRINT < right && position.x + HALF_FOOTPRINT > left
        }
    };
    if !transverse_overlap {
        return None;
    }
    match direction {
        Direction::East if position.x + HALF_FOOTPRINT <= left => {
            Some(left - HALF_FOOTPRINT - position.x)
        }
        Direction::West if position.x - HALF_FOOTPRINT >= right => {
            Some(position.x - HALF_FOOTPRINT - right)
        }
        Direction::South if position.y + HALF_FOOTPRINT <= top => {
            Some(top - HALF_FOOTPRINT - position.y)
        }
        Direction::North if position.y - HALF_FOOTPRINT >= bottom => {
            Some(position.y - HALF_FOOTPRINT - bottom)
        }
        _ => None,
    }
}

/// The piecewise path taken by one actor in a tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MovementTrace {
    start: Position,
    segments: [Option<MovementSegment>; 2],
    len: u8,
}

impl MovementTrace {
    pub(crate) const fn new(start: Position) -> Self {
        Self {
            start,
            segments: [None; 2],
            len: 0,
        }
    }

    pub(crate) const fn start(&self) -> Position {
        self.start
    }

    pub(crate) fn push(&mut self, segment: MovementSegment) {
        assert!(self.len < 2, "movement traces contain at most two segments");
        self.segments[usize::from(self.len)] = Some(segment);
        self.len += 1;
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn segments(&self) -> impl Iterator<Item = MovementSegment> + '_ {
        self.segments[..self.len()].iter().flatten().copied()
    }

    /// Whether any actual movement segment swept across a cell's interior.
    #[must_use]
    pub fn overlaps_cell(&self, cell: CellId) -> bool {
        if self.is_empty() {
            MovementSegment::new(self.start, self.start).overlaps_cell(cell)
        } else {
            self.segments().any(|segment| segment.overlaps_cell(cell))
        }
    }
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
