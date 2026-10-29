use crate::game::types::TICKS_PER_SECOND;

pub const MIN_LAIR_SPAWN_SECONDS: u8 = 5;
pub const MAX_LAIR_SPAWN_SECONDS: u8 = 60;
pub const DEFAULT_LAIR_SPAWN_SECONDS: u8 = 10;
pub const MIN_MOBS_PER_LAIR: u8 = 1;
pub const MAX_MOBS_PER_LAIR: u8 = 8;
pub const DEFAULT_MOBS_PER_LAIR: u8 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LairSettingsError {
    InvalidSpawnSeconds(u8),
    InvalidMobsPerLair(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LairSettings {
    spawn_seconds: u8,
    mobs_per_lair: u8,
}

impl LairSettings {
    pub const DEFAULT: Self = Self {
        spawn_seconds: DEFAULT_LAIR_SPAWN_SECONDS,
        mobs_per_lair: DEFAULT_MOBS_PER_LAIR,
    };

    pub const fn new(spawn_seconds: u8, mobs_per_lair: u8) -> Result<Self, LairSettingsError> {
        if spawn_seconds < MIN_LAIR_SPAWN_SECONDS || spawn_seconds > MAX_LAIR_SPAWN_SECONDS {
            return Err(LairSettingsError::InvalidSpawnSeconds(spawn_seconds));
        }
        if mobs_per_lair < MIN_MOBS_PER_LAIR || mobs_per_lair > MAX_MOBS_PER_LAIR {
            return Err(LairSettingsError::InvalidMobsPerLair(mobs_per_lair));
        }
        Ok(Self {
            spawn_seconds,
            mobs_per_lair,
        })
    }

    #[must_use]
    pub const fn spawn_seconds(self) -> u8 {
        self.spawn_seconds
    }

    #[must_use]
    pub const fn spawn_ticks(self) -> u64 {
        self.spawn_seconds as u64 * TICKS_PER_SECOND as u64
    }

    #[must_use]
    pub const fn mobs_per_lair(self) -> u8 {
        self.mobs_per_lair
    }
}

impl Default for LairSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LairConfig {
    Disabled,
    Enabled(LairSettings),
}

impl Default for LairConfig {
    fn default() -> Self {
        Self::Enabled(LairSettings::DEFAULT)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_bounds_are_explicit() {
        assert_eq!(
            LairConfig::default(),
            LairConfig::Enabled(LairSettings::DEFAULT)
        );
        assert_eq!(LairSettings::DEFAULT.spawn_ticks(), 600);
        assert!(LairSettings::new(5, 1).is_ok());
        assert!(LairSettings::new(60, 8).is_ok());
        assert_eq!(
            LairSettings::new(4, 3),
            Err(LairSettingsError::InvalidSpawnSeconds(4))
        );
        assert_eq!(
            LairSettings::new(10, 9),
            Err(LairSettingsError::InvalidMobsPerLair(9))
        );
    }
}
