use serde::{Serialize, Serializer};

use crate::protocol::InvalidScalar;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RulesVersion {
    #[default]
    ClassicV1,
    ClassicV2,
    ClassicV3,
    ConfigurableV4,
    ExpandedV5,
}

impl RulesVersion {
    #[must_use]
    pub const fn get(self) -> u16 {
        match self {
            Self::ClassicV1 => 1,
            Self::ClassicV2 => 2,
            Self::ClassicV3 => 3,
            Self::ConfigurableV4 => 4,
            Self::ExpandedV5 => 5,
        }
    }
}

impl TryFrom<u16> for RulesVersion {
    type Error = InvalidScalar;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::ClassicV1),
            2 => Ok(Self::ClassicV2),
            3 => Ok(Self::ClassicV3),
            4 => Ok(Self::ConfigurableV4),
            5 => Ok(Self::ExpandedV5),
            _ => Err(InvalidScalar),
        }
    }
}

impl Serialize for RulesVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.get())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub enum MapId {
    #[default]
    #[serde(rename = "fixed_v1")]
    FixedV1,
    #[serde(rename = "generated_v1")]
    GeneratedV1,
    #[serde(rename = "generated_v2")]
    GeneratedV2,
}
