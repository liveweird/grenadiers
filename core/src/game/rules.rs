use super::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, TURN_ASSIST, UNITS_PER_CELL};

/// Immutable parameters for the initial deterministic rules profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RulesConfig {
    version: u16,
    tick_rate: u16,
    units_per_cell: i32,
    half_footprint: i32,
    turn_assist: i32,
    bomb_capacity: u16,
    bomb_range: u16,
    bomb_fuse_ticks: u64,
    flame_ticks: u64,
    round_ticks: u64,
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
            bomb_capacity: 1,
            bomb_range: 2,
            bomb_fuse_ticks: 180,
            flame_ticks: 30,
            round_ticks: 10_800,
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

    #[must_use]
    pub const fn bomb_capacity(self) -> u16 {
        self.bomb_capacity
    }

    #[must_use]
    pub const fn bomb_range(self) -> u16 {
        self.bomb_range
    }

    #[must_use]
    pub const fn bomb_fuse_ticks(self) -> u64 {
        self.bomb_fuse_ticks
    }

    #[must_use]
    pub const fn flame_ticks(self) -> u64 {
        self.flame_ticks
    }

    #[must_use]
    pub const fn round_ticks(self) -> u64 {
        self.round_ticks
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
