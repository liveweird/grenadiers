use std::cmp::Ordering;

pub const UNITS_PER_CELL: i32 = 1_024;
pub const HALF_FOOTPRINT: i32 = 320;
pub const TURN_ASSIST: i32 = 128;
pub const TICKS_PER_SECOND: u16 = 60;

/// A row and column in the logical arena grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellId {
    pub column: u16,
    pub row: u16,
}

impl CellId {
    #[must_use]
    pub const fn new(column: u16, row: u16) -> Self {
        Self { column, row }
    }
}

impl Ord for CellId {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.row, self.column).cmp(&(other.row, other.column))
    }
}

impl PartialOrd for CellId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SeatId {
    One,
    Two,
}

impl SeatId {
    pub const ALL: [Self; 2] = [Self::One, Self::Two];

    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(pub u64);

/// A fixed-point position measured in 1/1024-cell units.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    #[must_use]
    pub fn center(cell: CellId) -> Self {
        Self {
            x: i32::from(cell.column) * UNITS_PER_CELL + UNITS_PER_CELL / 2,
            y: i32::from(cell.row) * UNITS_PER_CELL + UNITS_PER_CELL / 2,
        }
    }

    /// Returns the containing cell, or `None` when either coordinate cannot be
    /// represented by a non-negative `CellId` coordinate.
    #[must_use]
    pub fn cell(self) -> Option<CellId> {
        let column = u16::try_from(self.x.div_euclid(UNITS_PER_CELL)).ok()?;
        let row = u16::try_from(self.y.div_euclid(UNITS_PER_CELL)).ok()?;
        Some(CellId::new(column, row))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum SpeedLevel {
    #[default]
    Base,
    One,
    Two,
    Three,
    Four,
}

impl SpeedLevel {
    #[must_use]
    pub const fn units_per_second(self) -> u16 {
        match self {
            Self::Base => 3_072,
            Self::One => 3_584,
            Self::Two => 4_096,
            Self::Three => 4_608,
            Self::Four => 5_120,
        }
    }
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
