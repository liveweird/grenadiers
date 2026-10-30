use super::lairs::DEFAULT_LAIR_SPAWN_SECONDS;
use super::{LairConfig, LairSettings, LairSettingsError};

/// Rules9 lair tuning whose living-mob limit is fixed at one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RespawnLairSettings(LairSettings);

impl RespawnLairSettings {
    pub const DEFAULT: Self = match LairSettings::new(DEFAULT_LAIR_SPAWN_SECONDS, 1) {
        Ok(settings) => Self(settings),
        Err(_) => panic!("default respawn lair settings must be valid"),
    };

    pub const fn new(spawn_seconds: u8) -> Result<Self, LairSettingsError> {
        match LairSettings::new(spawn_seconds, 1) {
            Ok(settings) => Ok(Self(settings)),
            Err(error) => Err(error),
        }
    }

    #[must_use]
    pub const fn spawn_seconds(self) -> u8 {
        self.0.spawn_seconds()
    }

    #[must_use]
    pub const fn spawn_ticks(self) -> u64 {
        self.0.spawn_ticks()
    }

    pub(crate) const fn legacy(self) -> LairSettings {
        self.0
    }
}

impl Default for RespawnLairSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RespawnLairConfig {
    Disabled,
    Enabled(RespawnLairSettings),
}

impl RespawnLairConfig {
    pub(crate) const fn as_legacy(self) -> LairConfig {
        match self {
            Self::Disabled => LairConfig::Disabled,
            Self::Enabled(settings) => LairConfig::Enabled(settings.legacy()),
        }
    }
}

impl Default for RespawnLairConfig {
    fn default() -> Self {
        Self::Enabled(RespawnLairSettings::DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_delay_and_always_converts_to_a_cap_of_one() {
        let settings = RespawnLairSettings::new(5).unwrap();
        assert_eq!(settings.spawn_ticks(), 300);
        assert_eq!(settings.legacy().mobs_per_lair(), 1);
        assert_eq!(
            RespawnLairSettings::new(4),
            Err(LairSettingsError::InvalidSpawnSeconds(4))
        );
    }
}
