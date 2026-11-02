use serde::{Deserialize, Serialize};

use super::Environment;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomSettings {
    pub base: super::GameSettings,
    pub environment: Environment,
    bomb_capacity_drop_multiplier: crate::room::BombCapacityDropMultiplier,
}

impl RoomSettings {
    #[must_use]
    pub const fn new(
        base: super::GameSettings,
        environment: Environment,
        bomb_capacity_drop_multiplier: crate::room::BombCapacityDropMultiplier,
    ) -> Self {
        Self {
            base,
            environment,
            bomb_capacity_drop_multiplier,
        }
    }

    #[must_use]
    pub const fn bomb_capacity_drop_multiplier(self) -> crate::room::BombCapacityDropMultiplier {
        self.bomb_capacity_drop_multiplier
    }

    pub(crate) fn domain(self) -> Result<crate::room::RoomSettings, ()> {
        let speed = match self.base.starting_speed_level() {
            0 => crate::game::SpeedLevel::Base,
            1 => crate::game::SpeedLevel::One,
            2 => crate::game::SpeedLevel::Two,
            3 => crate::game::SpeedLevel::Three,
            4 => crate::game::SpeedLevel::Four,
            _ => return Err(()),
        };
        let enabled_items =
            crate::game::EnabledItems::from_bits(self.base.enabled_items().mask().bits())
                .ok_or(())?;
        crate::room::RoomSettings::new(
            speed,
            self.base.turn_assist_units(),
            self.base.wins_to_finish(),
            u64::from(self.base.bomb_fuse_ticks()),
        )
        .map(|settings| {
            settings
                .with_enabled_items(enabled_items)
                .with_bomb_capacity_drop_multiplier(self.bomb_capacity_drop_multiplier)
        })
        .map_err(|_| ())
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    starting_speed_level: u8,
    turn_assist_units: i32,
    wins_to_finish: u8,
    bomb_fuse_ticks: u16,
    bomb_capacity_drop_multiplier: u64,
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
        let multiplier =
            crate::room::BombCapacityDropMultiplier::new(fields.bomb_capacity_drop_multiplier)
                .map_err(|_| serde::de::Error::custom("invalid Bomb Capacity drop multiplier"))?;
        Ok(Self::new(base, fields.environment, multiplier))
    }
}

impl Serialize for RoomSettings {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Fields {
            starting_speed_level: self.base.starting_speed_level(),
            turn_assist_units: self.base.turn_assist_units(),
            wins_to_finish: self.base.wins_to_finish(),
            bomb_fuse_ticks: self.base.bomb_fuse_ticks(),
            bomb_capacity_drop_multiplier: self.bomb_capacity_drop_multiplier.value(),
            enabled_items: self.base.enabled_items(),
            environment: self.environment,
        }
        .serialize(serializer)
    }
}
