/// A logical movement direction, independent of any input device or key binding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

/// Tracks held directions and resolves simultaneous input by press recency.
///
/// A newly pressed direction becomes active. Repeated press events for a
/// direction that is already held leave its priority unchanged. When the active
/// direction is released, the most recently pressed remaining direction resumes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HeldDirections {
    directions: [Option<Direction>; 4],
    len: usize,
}

impl HeldDirections {
    /// Records a direction as held, giving a new press the highest priority.
    pub fn press(&mut self, direction: Direction) {
        if self.directions[..self.len].contains(&Some(direction)) {
            return;
        }

        // There are exactly four possible directions, and held directions are
        // unique, so a new direction always has room in this fixed-size array.
        self.directions[self.len] = Some(direction);
        self.len += 1;
    }

    /// Releases a direction if it is held, preserving the order of the others.
    pub fn release(&mut self, direction: Direction) {
        let Some(index) = self.directions[..self.len]
            .iter()
            .position(|held| *held == Some(direction))
        else {
            return;
        };

        self.directions.copy_within(index + 1..self.len, index);
        self.len -= 1;
        self.directions[self.len] = None;
    }

    /// Releases every held direction.
    pub fn clear(&mut self) {
        self.directions = [None; 4];
        self.len = 0;
    }

    /// Returns the most recently pressed direction that remains held.
    #[must_use]
    pub fn active(&self) -> Option<Direction> {
        self.len
            .checked_sub(1)
            .and_then(|index| self.directions[index])
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod tests;
