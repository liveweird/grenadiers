//! Environment-aware room configuration for dormant rules-v8 gameplay.

use crate::game::StormCloudConfig;

use super::{LairConfig, LairSettings, RoomConfiguration};

/// Wire-friendly lair choices that retain their numeric values while disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LairTuning {
    enabled: bool,
    settings: LairSettings,
}

impl LairTuning {
    #[must_use]
    pub const fn new(enabled: bool, settings: LairSettings) -> Self {
        Self { enabled, settings }
    }

    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn settings(self) -> LairSettings {
        self.settings
    }

    #[must_use]
    pub const fn config(self) -> LairConfig {
        if self.enabled {
            LairConfig::Enabled(self.settings)
        } else {
            LairConfig::Disabled
        }
    }
}

/// A closed rules-v8 configuration without widening earlier room contracts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LairRoomConfiguration {
    base: RoomConfiguration,
    storm_cloud: StormCloudConfig,
    lairs: LairTuning,
}

impl LairRoomConfiguration {
    #[must_use]
    pub const fn new(
        base: RoomConfiguration,
        storm_cloud: StormCloudConfig,
        lairs: LairTuning,
    ) -> Self {
        Self {
            base,
            storm_cloud,
            lairs,
        }
    }

    #[must_use]
    pub const fn base(&self) -> &RoomConfiguration {
        &self.base
    }

    #[must_use]
    pub const fn storm_cloud(&self) -> StormCloudConfig {
        self.storm_cloud
    }

    #[must_use]
    pub const fn lairs(&self) -> LairTuning {
        self.lairs
    }
}
