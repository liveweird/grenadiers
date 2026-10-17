use super::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, TURN_ASSIST, UNITS_PER_CELL};

const FORGIVING_TURN_ASSIST: i32 = 256;

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
    pickup_collection: bool,
    bomb_capacity_cap: u16,
    bomb_range_cap: u16,
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
            pickup_collection: false,
            bomb_capacity_cap: 8,
            bomb_range_cap: 8,
            bomb_fuse_ticks: 180,
            flame_ticks: 30,
            round_ticks: 10_800,
        }
    }

    #[must_use]
    pub const fn classic_v2() -> Self {
        Self {
            version: 2,
            pickup_collection: true,
            ..Self::classic_v1()
        }
    }

    /// The generated-arena profile with a wider lane-centering window.
    ///
    /// Earlier profiles retain their original turn tolerance so recorded input
    /// streams continue to produce the same movement traces.
    #[must_use]
    pub const fn classic_v3() -> Self {
        Self {
            version: 3,
            turn_assist: FORGIVING_TURN_ASSIST,
            ..Self::classic_v2()
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
    pub const fn pickup_collection(self) -> bool {
        self.pickup_collection
    }

    #[must_use]
    pub const fn bomb_capacity_cap(self) -> u16 {
        self.bomb_capacity_cap
    }

    #[must_use]
    pub const fn bomb_range_cap(self) -> u16 {
        self.bomb_range_cap
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
