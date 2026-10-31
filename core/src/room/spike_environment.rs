//! Rules-v10 room configuration with independently retained spike-pit tuning.

use super::{LairRoomConfiguration, SpikePitConfig, SpikePitSettings};

/// Wire-friendly spike-pit choices that retain durations while disabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpikePitTuning {
    enabled: bool,
    settings: SpikePitSettings,
}

impl SpikePitTuning {
    pub const DEFAULT: Self = Self::new(true, SpikePitSettings::DEFAULT);

    #[must_use]
    pub const fn new(enabled: bool, settings: SpikePitSettings) -> Self {
        Self { enabled, settings }
    }

    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn settings(self) -> SpikePitSettings {
        self.settings
    }

    #[must_use]
    pub const fn config(self) -> SpikePitConfig {
        if self.enabled {
            SpikePitConfig::Enabled(self.settings)
        } else {
            SpikePitConfig::Disabled
        }
    }
}

impl Default for SpikePitTuning {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// A closed rules-v10 configuration without widening the rules-v9 domain shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpikeRoomConfiguration {
    base: LairRoomConfiguration,
    spike_pits: SpikePitTuning,
}

impl SpikeRoomConfiguration {
    #[must_use]
    pub const fn new(base: LairRoomConfiguration, spike_pits: SpikePitTuning) -> Self {
        Self { base, spike_pits }
    }

    #[must_use]
    pub const fn base(&self) -> &LairRoomConfiguration {
        &self.base
    }

    #[must_use]
    pub const fn spike_pits(&self) -> SpikePitTuning {
        self.spike_pits
    }
}

impl From<LairRoomConfiguration> for SpikeRoomConfiguration {
    fn from(base: LairRoomConfiguration) -> Self {
        Self::new(base, SpikePitTuning::DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_tuning_retains_checked_durations() {
        let settings = SpikePitSettings::new(12, 240).unwrap();
        let tuning = SpikePitTuning::new(false, settings);
        assert_eq!(tuning.config(), SpikePitConfig::Disabled);
        assert_eq!(tuning.settings(), settings);
        assert!(SpikePitTuning::default().enabled());
    }
}
