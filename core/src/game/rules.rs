use super::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, TURN_ASSIST, UNITS_PER_CELL};

/// Immutable parameters for the initial deterministic rules profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RulesConfig {
    version: u16,
    tick_rate: u16,
    units_per_cell: i32,
    half_footprint: i32,
    turn_assist: i32,
}

impl RulesConfig {
    #[must_use]
    pub const fn classic_v1() -> Self {
        Self {
            version: 1,
            tick_rate: TICKS_PER_SECOND,
            units_per_cell: UNITS_PER_CELL,
            half_footprint: HALF_FOOTPRINT,
            turn_assist: TURN_ASSIST,
        }
    }

    #[must_use]
    pub const fn version(self) -> u16 {
        self.version
    }

    #[must_use]
    pub const fn tick_rate(self) -> u16 {
        self.tick_rate
    }

    #[must_use]
    pub const fn units_per_cell(self) -> i32 {
        self.units_per_cell
    }

    #[must_use]
    pub const fn half_footprint(self) -> i32 {
        self.half_footprint
    }

    #[must_use]
    pub const fn turn_assist(self) -> i32 {
        self.turn_assist
    }
}

impl Default for RulesConfig {
    fn default() -> Self {
        Self::classic_v1()
    }
}

#[cfg(test)]
#[path = "rules_tests.rs"]
mod tests;
