//! Rules9 tuning retains a respawn delay while disabled and fixes capacity at one.

use super::{LairSettingsError, LairTuning, RespawnLairConfig, RespawnLairSettings};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RespawnLairTuning {
    enabled: bool,
    settings: RespawnLairSettings,
}

impl RespawnLairTuning {
    pub const DEFAULT: Self = Self::new(true, RespawnLairSettings::DEFAULT);

    #[must_use]
    pub const fn new(enabled: bool, settings: RespawnLairSettings) -> Self {
        Self { enabled, settings }
    }

    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn settings(self) -> RespawnLairSettings {
        self.settings
    }

    #[must_use]
    pub const fn config(self) -> RespawnLairConfig {
        if self.enabled {
            RespawnLairConfig::Enabled(self.settings)
        } else {
            RespawnLairConfig::Disabled
        }
    }

    #[must_use]
    pub const fn as_legacy(self) -> LairTuning {
        LairTuning::new(self.enabled, self.settings.legacy())
    }
}

impl TryFrom<LairTuning> for RespawnLairTuning {
    type Error = LairSettingsError;

    fn try_from(value: LairTuning) -> Result<Self, Self::Error> {
        if value.settings().mobs_per_lair() != 1 {
            return Err(LairSettingsError::InvalidMobsPerLair(
                value.settings().mobs_per_lair(),
            ));
        }
        Ok(Self::new(
            value.enabled(),
            RespawnLairSettings::new(value.settings().spawn_seconds())?,
        ))
    }
}

impl Default for RespawnLairTuning {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::room::LairSettings;

    #[test]
    fn disabled_tuning_retains_delay_without_accepting_legacy_capacity() {
        let disabled = RespawnLairTuning::new(false, RespawnLairSettings::new(17).unwrap());
        assert_eq!(disabled.config(), RespawnLairConfig::Disabled);
        assert_eq!(disabled.as_legacy().settings().spawn_seconds(), 17);
        assert_eq!(
            RespawnLairTuning::try_from(disabled.as_legacy()),
            Ok(disabled)
        );
        assert!(
            RespawnLairTuning::try_from(LairTuning::new(false, LairSettings::DEFAULT)).is_err()
        );
    }
}
