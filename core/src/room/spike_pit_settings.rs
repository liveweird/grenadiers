pub const MIN_SPIKE_PIT_WARNING_TICKS: u64 = 6;
pub const MAX_SPIKE_PIT_WARNING_TICKS: u64 = 120;
pub const DEFAULT_SPIKE_PIT_WARNING_TICKS: u16 = 36;
pub const MIN_SPIKE_PIT_RAISED_TICKS: u64 = 60;
pub const MAX_SPIKE_PIT_RAISED_TICKS: u64 = 600;
pub const DEFAULT_SPIKE_PIT_RAISED_TICKS: u16 = 180;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpikePitSettingsError {
    InvalidWarningTicks(u64),
    InvalidRaisedTicks(u64),
}

impl std::fmt::Display for SpikePitSettingsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for SpikePitSettingsError {}

/// Checked warning and raised durations for one room's spike pits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpikePitSettings {
    warning_ticks: u16,
    raised_ticks: u16,
}

impl SpikePitSettings {
    pub const DEFAULT: Self = Self {
        warning_ticks: DEFAULT_SPIKE_PIT_WARNING_TICKS,
        raised_ticks: DEFAULT_SPIKE_PIT_RAISED_TICKS,
    };

    pub const fn new(warning_ticks: u64, raised_ticks: u64) -> Result<Self, SpikePitSettingsError> {
        if warning_ticks < MIN_SPIKE_PIT_WARNING_TICKS
            || warning_ticks > MAX_SPIKE_PIT_WARNING_TICKS
        {
            return Err(SpikePitSettingsError::InvalidWarningTicks(warning_ticks));
        }
        if raised_ticks < MIN_SPIKE_PIT_RAISED_TICKS || raised_ticks > MAX_SPIKE_PIT_RAISED_TICKS {
            return Err(SpikePitSettingsError::InvalidRaisedTicks(raised_ticks));
        }
        Ok(Self {
            warning_ticks: warning_ticks as u16,
            raised_ticks: raised_ticks as u16,
        })
    }

    #[must_use]
    pub const fn warning_ticks(self) -> u64 {
        self.warning_ticks as u64
    }

    #[must_use]
    pub const fn raised_ticks(self) -> u64 {
        self.raised_ticks as u64
    }
}

impl Default for SpikePitSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Dormant configuration boundary for a future live rules profile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpikePitConfig {
    #[default]
    Disabled,
    Enabled(SpikePitSettings),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_sixty_hertz_product_values_and_config_stays_disabled() {
        assert_eq!(SpikePitSettings::DEFAULT.warning_ticks(), 36);
        assert_eq!(SpikePitSettings::DEFAULT.raised_ticks(), 180);
        assert_eq!(SpikePitConfig::default(), SpikePitConfig::Disabled);
    }

    #[test]
    fn exact_tick_bounds_are_inclusive() {
        assert!(SpikePitSettings::new(6, 60).is_ok());
        assert!(SpikePitSettings::new(120, 600).is_ok());
        assert_eq!(
            SpikePitSettings::new(5, 180),
            Err(SpikePitSettingsError::InvalidWarningTicks(5))
        );
        assert_eq!(
            SpikePitSettings::new(36, 601),
            Err(SpikePitSettingsError::InvalidRaisedTicks(601))
        );
    }
}
