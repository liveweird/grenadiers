//! Environment-aware room configuration for the dormant rules-v7 profile.

use crate::game::StormCloudConfig;

use super::RoomConfiguration;

/// A room configuration paired with its immutable round environment profile.
///
/// Keeping this wrapper separate preserves the meaning and constructors of the
/// closed pre-environment [`RoomConfiguration`] contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnvironmentRoomConfiguration {
    base: RoomConfiguration,
    storm_cloud: StormCloudConfig,
}

impl EnvironmentRoomConfiguration {
    #[must_use]
    pub const fn new(base: RoomConfiguration, storm_cloud: StormCloudConfig) -> Self {
        Self { base, storm_cloud }
    }

    #[must_use]
    pub const fn base(&self) -> &RoomConfiguration {
        &self.base
    }

    #[must_use]
    pub const fn storm_cloud(&self) -> StormCloudConfig {
        self.storm_cloud
    }
}
