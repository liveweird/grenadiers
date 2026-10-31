use serde::{Deserialize, Serialize};

use crate::game::StormCloudConfig;
use crate::room::{
    LairRoomConfiguration as DomainConfiguration, LairSettingsError, RespawnLairSettings,
    RespawnLairTuning,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Lairs {
    pub enabled: bool,
    pub spawn_seconds: u8,
    pub max_living_mobs: u8,
}

impl Lairs {
    fn domain(self) -> Result<RespawnLairTuning, LairSettingsError> {
        if self.max_living_mobs != 1 {
            return Err(LairSettingsError::InvalidMobsPerLair(self.max_living_mobs));
        }
        Ok(RespawnLairTuning::new(
            self.enabled,
            RespawnLairSettings::new(self.spawn_seconds)?,
        ))
    }

    const fn from_domain(value: RespawnLairTuning) -> Self {
        Self {
            enabled: value.enabled(),
            spawn_seconds: value.settings().spawn_seconds(),
            max_living_mobs: 1,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Environment {
    pub storm_cloud: bool,
    pub lairs: Lairs,
}

impl Environment {
    pub(crate) fn domain(self) -> Result<(StormCloudConfig, RespawnLairTuning), LairSettingsError> {
        Ok((
            if self.storm_cloud {
                StormCloudConfig::Single
            } else {
                StormCloudConfig::Disabled
            },
            self.lairs.domain()?,
        ))
    }

    const fn from_domain(cloud: StormCloudConfig, lairs: RespawnLairTuning) -> Self {
        Self {
            storm_cloud: matches!(cloud, StormCloudConfig::Single),
            lairs: Lairs::from_domain(lairs),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fields {
    controllers: Vec<super::ConfigurationController>,
    #[serde(deserialize_with = "required_optional")]
    cpu_difficulty: Option<super::Difficulty>,
    board: super::BoardSize,
    environment: Environment,
}

fn required_optional<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<super::Difficulty>, D::Error> {
    Option::deserialize(deserializer)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomConfiguration {
    domain: DomainConfiguration,
}

impl RoomConfiguration {
    #[must_use]
    pub const fn domain(&self) -> &DomainConfiguration {
        &self.domain
    }
    pub fn from_domain(domain: DomainConfiguration) -> Result<Self, LairSettingsError> {
        RespawnLairTuning::try_from(domain.lairs())?;
        Ok(Self { domain })
    }
}

impl<'de> Deserialize<'de> for RoomConfiguration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = Fields::deserialize(deserializer)?;
        let value = serde_json::to_value(&fields).map_err(serde::de::Error::custom)?;
        let mut base = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("invalid configuration"))?
            .clone();
        base.remove("environment");
        let base: crate::protocol::v14::RoomConfiguration =
            serde_json::from_value(base.into()).map_err(serde::de::Error::custom)?;
        let (cloud, lairs) = fields
            .environment
            .domain()
            .map_err(|_| serde::de::Error::custom("invalid lair settings"))?;
        Ok(Self {
            domain: DomainConfiguration::new(*base.domain(), cloud, lairs.as_legacy()),
        })
    }
}

impl Serialize for RoomConfiguration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Borrowed {
            #[serde(flatten)]
            base: crate::protocol::v14::RoomConfiguration,
            environment: Environment,
        }
        Borrowed {
            base: crate::protocol::v14::RoomConfiguration::from_domain(*self.domain.base()),
            environment: Environment::from_domain(
                self.domain.storm_cloud(),
                RespawnLairTuning::try_from(self.domain.lairs())
                    .expect("validated respawn configuration"),
            ),
        }
        .serialize(serializer)
    }
}
