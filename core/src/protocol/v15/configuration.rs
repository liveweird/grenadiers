use serde::{Deserialize, Serialize};

use crate::game::StormCloudConfig;
use crate::room::EnvironmentRoomConfiguration as DomainConfiguration;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Environment {
    pub storm_cloud: bool,
}

impl Environment {
    pub(crate) const fn domain(self) -> StormCloudConfig {
        if self.storm_cloud {
            StormCloudConfig::Single
        } else {
            StormCloudConfig::Disabled
        }
    }

    pub(crate) const fn from_domain(value: StormCloudConfig) -> Self {
        Self {
            storm_cloud: matches!(value, StormCloudConfig::Single),
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

    #[must_use]
    pub const fn from_domain(domain: DomainConfiguration) -> Self {
        Self { domain }
    }
}

impl<'de> Deserialize<'de> for RoomConfiguration {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let fields = Fields::deserialize(deserializer)?;
        let value = serde_json::to_value(&fields).map_err(serde::de::Error::custom)?;
        let object = value
            .as_object()
            .ok_or_else(|| serde::de::Error::custom("invalid configuration"))?;
        let mut base = object.clone();
        base.remove("environment");
        let base: crate::protocol::v14::RoomConfiguration =
            serde_json::from_value(base.into()).map_err(serde::de::Error::custom)?;
        Ok(Self {
            domain: DomainConfiguration::new(*base.domain(), fields.environment.domain()),
        })
    }
}

impl Serialize for RoomConfiguration {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Borrowed<'a> {
            #[serde(flatten)]
            base: crate::protocol::v14::RoomConfiguration,
            environment: Environment,
            #[serde(skip)]
            marker: std::marker::PhantomData<&'a ()>,
        }
        Borrowed {
            base: crate::protocol::v14::RoomConfiguration::from_domain(*self.domain.base()),
            environment: Environment::from_domain(self.domain.storm_cloud()),
            marker: std::marker::PhantomData,
        }
        .serialize(serializer)
    }
}
