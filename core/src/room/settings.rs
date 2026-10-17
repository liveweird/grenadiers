use crate::game::SpeedLevel;

pub const MIN_TURN_ASSIST_UNITS: i32 = 64;
pub const MAX_TURN_ASSIST_UNITS: i32 = 256;
pub const TURN_ASSIST_STEP_UNITS: i32 = 64;
pub const MIN_WINS_TO_FINISH: u8 = 1;
pub const MAX_WINS_TO_FINISH: u8 = 9;
pub const MIN_BOMB_FUSE_TICKS: u64 = 60;
pub const MAX_BOMB_FUSE_TICKS: u64 = 300;
pub const BOMB_FUSE_STEP_TICKS: u64 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsError {
    InvalidTurnAssistUnits(i32),
    InvalidWinsToFinish(u8),
    InvalidBombFuseTicks(u64),
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for SettingsError {}

/// Validated, immutable gameplay choices for one room and its rematches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomSettings {
    starting_speed: SpeedLevel,
    turn_assist_units: i32,
    wins_to_finish: u8,
    bomb_fuse_ticks: u64,
}

impl RoomSettings {
    pub const DEFAULT: Self = Self {
        starting_speed: SpeedLevel::Base,
        turn_assist_units: MAX_TURN_ASSIST_UNITS,
        wins_to_finish: 3,
        bomb_fuse_ticks: 180,
    };

    pub fn new(
        starting_speed: SpeedLevel,
        turn_assist_units: i32,
        wins_to_finish: u8,
        bomb_fuse_ticks: u64,
    ) -> Result<Self, SettingsError> {
        if !(MIN_TURN_ASSIST_UNITS..=MAX_TURN_ASSIST_UNITS).contains(&turn_assist_units)
            || turn_assist_units % TURN_ASSIST_STEP_UNITS != 0
        {
            return Err(SettingsError::InvalidTurnAssistUnits(turn_assist_units));
        }
        if !(MIN_WINS_TO_FINISH..=MAX_WINS_TO_FINISH).contains(&wins_to_finish) {
            return Err(SettingsError::InvalidWinsToFinish(wins_to_finish));
        }
        if !(MIN_BOMB_FUSE_TICKS..=MAX_BOMB_FUSE_TICKS).contains(&bomb_fuse_ticks)
            || !bomb_fuse_ticks.is_multiple_of(BOMB_FUSE_STEP_TICKS)
        {
            return Err(SettingsError::InvalidBombFuseTicks(bomb_fuse_ticks));
        }
        Ok(Self {
            starting_speed,
            turn_assist_units,
            wins_to_finish,
            bomb_fuse_ticks,
        })
    }

    #[must_use]
    pub const fn starting_speed(self) -> SpeedLevel {
        self.starting_speed
    }

    #[must_use]
    pub const fn turn_assist_units(self) -> i32 {
        self.turn_assist_units
    }

    #[must_use]
    pub const fn wins_to_finish(self) -> u8 {
        self.wins_to_finish
    }

    #[must_use]
    pub const fn bomb_fuse_ticks(self) -> u64 {
        self.bomb_fuse_ticks
    }
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;
