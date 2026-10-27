use std::num::NonZeroU32;

use super::arena::Arena;
use super::types::{CellId, PlayerCount, SeatId};

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
        self.priority_seat_for(PlayerCount::Two)
    }

    #[must_use]
    pub const fn priority_seat_for(self, count: PlayerCount) -> SeatId {
        count.seats()[((self.round.get() - 1) as usize) % count.get()]
    }

    pub fn priority_order(self, count: PlayerCount) -> impl Iterator<Item = SeatId> {
        let start = ((self.round.get() - 1) as usize) % count.get();
        (0..count.get()).map(move |offset| count.seats()[(start + offset) % count.get()])
    }

    /// Rotates seats across this round's supplied spawn cells in row-major order.
    #[must_use]
    pub fn spawns(self, arena: &Arena) -> Vec<CellId> {
        let mut spawns = arena.spawns().to_vec();
        spawns.sort();
        let rotation = ((self.round.get() - 1) as usize) % spawns.len();
        spawns.rotate_left(rotation);
        spawns
    }

    #[must_use]
    pub fn spawn_for(self, arena: &Arena, seat: SeatId) -> Option<CellId> {
        self.spawns(arena).get(seat.index()).copied()
    }
}

#[cfg(test)]
#[path = "assignment_tests.rs"]
mod tests;
