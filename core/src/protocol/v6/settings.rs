use serde::{Deserialize, Serialize};

use crate::game::SpeedLevel;
use crate::room::settings::RoomSettings;

use super::super::RulesVersion;

/// A complete, checked room settings value; partial updates are never accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Fields", into = "Fields")]
pub struct GameSettings(RoomSettings);

impl GameSettings {
    pub const fn from_domain(settings: RoomSettings) -> Result<Self, super::super::InvalidScalar> {
        if settings.enabled_items().bits() == 0 {
            Ok(Self(settings))
        } else {
            Err(super::super::InvalidScalar)
        }
    }

    #[must_use]
    pub const fn domain(self) -> RoomSettings {
        self.0
    }

    #[must_use]
    pub fn for_rules(version: RulesVersion) -> Self {
        let tolerance = match version {
            RulesVersion::ClassicV1 | RulesVersion::ClassicV2 => 128,
            RulesVersion::ClassicV3 | RulesVersion::ConfigurableV4 => 256,
        };
        Self(RoomSettings::new(SpeedLevel::Base, tolerance, 3, 180).expect("valid defaults"))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    #[serde(deserialize_with = "speed")]
    starting_speed_level: u8,
    #[serde(deserialize_with = "tolerance")]
    turn_assist_units: i32,
    #[serde(deserialize_with = "wins")]
    wins_to_finish: u8,
    #[serde(deserialize_with = "fuse")]
    bomb_fuse_ticks: u64,
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
bounded_input!(fuse, u64, 300);

impl From<GameSettings> for Fields {
    fn from(settings: GameSettings) -> Self {
        let settings = settings.0;
        Self {
            starting_speed_level: match settings.starting_speed() {
                SpeedLevel::Base => 0,
                SpeedLevel::One => 1,
                SpeedLevel::Two => 2,
                SpeedLevel::Three => 3,
                SpeedLevel::Four => 4,
            },
            turn_assist_units: settings.turn_assist_units(),
            wins_to_finish: settings.wins_to_finish(),
            bomb_fuse_ticks: settings.bomb_fuse_ticks(),
        }
    }
}

impl TryFrom<Fields> for GameSettings {
    type Error = super::super::InvalidScalar;

    fn try_from(fields: Fields) -> Result<Self, Self::Error> {
        let speed = match fields.starting_speed_level {
            0 => SpeedLevel::Base,
            1 => SpeedLevel::One,
            2 => SpeedLevel::Two,
            3 => SpeedLevel::Three,
            4 => SpeedLevel::Four,
            _ => return Err(super::super::InvalidScalar),
        };
        RoomSettings::new(
            speed,
            fields.turn_assist_units,
            fields.wins_to_finish,
            fields.bomb_fuse_ticks,
        )
        .map(Self)
        .map_err(|_| super::super::InvalidScalar)
    }
}
