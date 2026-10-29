use super::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, TURN_ASSIST, UNITS_PER_CELL};
use crate::room::{LairConfig, RoomSettings};

const FORGIVING_TURN_ASSIST: i32 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StormCloudConfig {
    Disabled,
    Single,
}

/// Immutable parameters for the initial deterministic rules profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RulesConfig {
    room_settings: Option<RoomSettings>,
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
    storm_cloud_config: StormCloudConfig,
    lair_config: LairConfig,
}

impl RulesConfig {
    #[must_use]
    pub const fn classic_v1() -> Self {
        Self {
            room_settings: None,
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
            storm_cloud_config: StormCloudConfig::Disabled,
            lair_config: LairConfig::Disabled,
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

    /// The configurable room profile. Validation is owned by `RoomSettings`.
    #[must_use]
    pub const fn configurable_v4(settings: RoomSettings) -> Self {
        Self {
            room_settings: Some(settings),
            version: 4,
            turn_assist: settings.turn_assist_units(),
            bomb_fuse_ticks: settings.bomb_fuse_ticks(),
            ..Self::classic_v3()
        }
    }

    /// Expanded room profile with its item set bound into the rules identity.
    #[must_use]
    pub const fn expanded_v5(settings: RoomSettings) -> Self {
        Self {
            version: 5,
            ..Self::configurable_v4(settings)
        }
    }

    /// Dormant headless multiplayer profile. Live room activation is intentionally separate.
    #[must_use]
    pub const fn multiplayer_v6(settings: RoomSettings) -> Self {
        Self {
            version: 6,
            ..Self::expanded_v5(settings)
        }
    }

    #[must_use]
    pub const fn environment_v7(settings: RoomSettings, clouds: StormCloudConfig) -> Self {
        Self {
            version: 7,
            storm_cloud_config: clouds,
            ..Self::multiplayer_v6(settings)
        }
    }

    /// Dormant environment profile with deterministic lairs and mobs.
    #[must_use]
    pub const fn environment_v8(
        settings: RoomSettings,
        clouds: StormCloudConfig,
        lairs: LairConfig,
    ) -> Self {
        Self {
            version: 8,
            lair_config: lairs,
            ..Self::environment_v7(settings, clouds)
        }
    }

    #[must_use]
    pub const fn lair_config(self) -> LairConfig {
        self.lair_config
    }

    #[must_use]
    pub const fn storm_cloud_config(self) -> StormCloudConfig {
        self.storm_cloud_config
    }

    #[must_use]
    pub const fn enabled_items(self) -> crate::game::EnabledItems {
        match self.room_settings {
            Some(settings) if self.version >= 5 => settings.enabled_items(),
            _ => crate::game::EnabledItems::EMPTY,
        }
    }

    #[must_use]
    pub const fn version(self) -> u16 {
        self.version
    }

    /// Complete immutable settings identity for configurable rounds.
    #[must_use]
    pub const fn room_settings(self) -> Option<RoomSettings> {
        self.room_settings
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
