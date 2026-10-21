use super::types::{CellId, HALF_FOOTPRINT, Position, UNITS_PER_CELL};

/// One axis-aligned piece of an actor's movement during a tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MovementSegment {
    start: Position,
    end: Position,
}

impl MovementSegment {
    #[must_use]
    pub(crate) const fn new(start: Position, end: Position) -> Self {
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
