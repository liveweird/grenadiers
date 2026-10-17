use std::num::NonZeroU32;

use super::arena::Arena;
use super::types::{CellId, SeatId};

/// Deterministic seat roles for one nonzero match round number.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoundAssignment {
    round: NonZeroU32,
}

impl RoundAssignment {
    #[must_use]
    pub const fn new(round: NonZeroU32) -> Self {
        Self { round }
    }

    #[must_use]
    pub const fn first() -> Self {
        Self {
            round: NonZeroU32::MIN,
        }
    }

    #[must_use]
    pub const fn round_number(self) -> NonZeroU32 {
        self.round
    }

    #[must_use]
    pub const fn priority_seat(self) -> SeatId {
        if self.round.get() % 2 == 1 {
            SeatId::One
        } else {
            SeatId::Two
        }
    }

    /// Maps seats to the arena's row-major first and second spawn cells.
    #[must_use]
    pub fn spawns(self, arena: &Arena) -> [CellId; 2] {
        let mut spawns = arena.spawns();
        spawns.sort();
        if self.priority_seat() == SeatId::Two {
            spawns.swap(0, 1);
        }
        spawns
    }

    #[must_use]
    pub fn spawn_for(self, arena: &Arena, seat: SeatId) -> CellId {
        self.spawns(arena)[seat.index()]
    }
}

#[cfg(test)]
#[path = "assignment_tests.rs"]
mod tests;
