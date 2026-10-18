use serde::{Deserialize, Serialize};

use super::super::RulesVersion;
use super::ItemSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields", into = "Fields")]
pub struct GameSettings {
    starting_speed_level: u8,
    turn_assist_units: i32,
    wins_to_finish: u8,
    bomb_fuse_ticks: u16,
    enabled_items: ItemSet,
}

impl GameSettings {
    pub const fn new(
        starting_speed_level: u8,
        turn_assist_units: i32,
        wins_to_finish: u8,
        bomb_fuse_ticks: u16,
        enabled_items: ItemSet,
    ) -> Result<Self, super::super::InvalidScalar> {
        let valid_tolerance = matches!(turn_assist_units, 64 | 128 | 192 | 256);
        let valid_fuse =
            bomb_fuse_ticks >= 60 && bomb_fuse_ticks <= 300 && bomb_fuse_ticks.is_multiple_of(30);
        if starting_speed_level <= 4
            && valid_tolerance
            && (wins_to_finish >= 1 && wins_to_finish <= 9)
            && valid_fuse
        {
            Ok(Self {
                starting_speed_level,
                turn_assist_units,
                wins_to_finish,
                bomb_fuse_ticks,
                enabled_items,
            })
        } else {
            Err(super::super::InvalidScalar)
        }
    }

    #[must_use]
    pub const fn for_rules(version: RulesVersion) -> Self {
        let tolerance = match version {
            RulesVersion::ClassicV1 | RulesVersion::ClassicV2 => 128,
            RulesVersion::ClassicV3 | RulesVersion::ConfigurableV4 => 256,
        };
        Self {
            starting_speed_level: 0,
            turn_assist_units: tolerance,
            wins_to_finish: 3,
            bomb_fuse_ticks: 180,
            enabled_items: ItemSet::empty(),
        }
    }

    #[must_use]
    pub const fn enabled_items(self) -> ItemSet {
        self.enabled_items
    }

    #[must_use]
    pub const fn starting_speed_level(self) -> u8 {
        self.starting_speed_level
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
    pub const fn bomb_fuse_ticks(self) -> u16 {
        self.bomb_fuse_ticks
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    #[serde(deserialize_with = "speed")]
    starting_speed_level: u8,
    #[serde(deserialize_with = "tolerance")]
    turn_assist_units: i32,
    #[serde(deserialize_with = "wins")]
    wins_to_finish: u8,
    #[serde(deserialize_with = "fuse")]
    bomb_fuse_ticks: u16,
    enabled_items: ItemSet,
}

macro_rules! bounded_input {
    ($name:ident, $storage:ty, $maximum:expr) => {
        fn $name<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<$storage, D::Error> {
            super::scalars::deserialize_bounded(deserializer, $maximum, "invalid gameplay setting")
                .map(|value| value as $storage)
        }
    };
}
bounded_input!(speed, u8, 4);
bounded_input!(tolerance, i32, 256);
bounded_input!(wins, u8, 9);
bounded_input!(fuse, u16, 300);

impl TryFrom<Fields> for GameSettings {
    type Error = super::super::InvalidScalar;
    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        Self::new(
            fields.starting_speed_level,
            fields.turn_assist_units,
            fields.wins_to_finish,
            fields.bomb_fuse_ticks,
            fields.enabled_items,
        )
    }
}
impl From<GameSettings> for Fields {
    fn from(settings: GameSettings) -> Self {
        Self {
            starting_speed_level: settings.starting_speed_level,
            turn_assist_units: settings.turn_assist_units,
            wins_to_finish: settings.wins_to_finish,
            bomb_fuse_ticks: settings.bomb_fuse_ticks,
            enabled_items: settings.enabled_items,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsContextError {
    UnavailableItem,
}

/// Checks the room-context rule that JSON Schema cannot express.
pub fn validate_settings_context(
    settings: &GameSettings,
    available_items: ItemSet,
) -> Result<(), SettingsContextError> {
    if settings.enabled_items.is_subset_of(available_items) {
        Ok(())
    } else {
        Err(SettingsContextError::UnavailableItem)
    }
}
