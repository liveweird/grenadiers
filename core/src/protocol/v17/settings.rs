use serde::{Deserialize, Serialize};

use super::Environment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomSettings {
    pub base: super::GameSettings,
    pub environment: Environment,
}

impl RoomSettings {
    #[must_use]
    pub const fn new(base: super::GameSettings, environment: Environment) -> Self {
        Self { base, environment }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    starting_speed_level: u8,
    turn_assist_units: i32,
    wins_to_finish: u8,
    bomb_fuse_ticks: u16,
    enabled_items: super::ItemSet,
    environment: Environment,
}

impl<'de> Deserialize<'de> for RoomSettings {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = Fields::deserialize(deserializer)?;
        let base = super::GameSettings::new(
            fields.starting_speed_level,
            fields.turn_assist_units,
            fields.wins_to_finish,
            fields.bomb_fuse_ticks,
            fields.enabled_items,
        )
        .map_err(|_| serde::de::Error::custom("invalid gameplay setting"))?;
        fields
            .environment
            .domain()
            .map_err(|_| serde::de::Error::custom("invalid lair settings"))?;
        Ok(Self::new(base, fields.environment))
    }
}

impl Serialize for RoomSettings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Fields {
            starting_speed_level: self.base.starting_speed_level(),
            turn_assist_units: self.base.turn_assist_units(),
            wins_to_finish: self.base.wins_to_finish(),
            bomb_fuse_ticks: self.base.bomb_fuse_ticks(),
            enabled_items: self.base.enabled_items(),
            environment: self.environment,
        }
        .serialize(serializer)
    }
}
