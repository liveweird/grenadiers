use serde::Serialize;

use super::super::InvalidScalar;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct RulesVersion(u8);

impl RulesVersion {
    pub const CLASSIC_V1: Self = Self(1);
    pub const REMATCH_V2: Self = Self(2);
    pub const CONFIGURABLE_V3: Self = Self(3);
    pub const CONFIGURABLE_V4: Self = Self(4);
    pub const EXPANDED_V5: Self = Self(5);
    pub const MULTIPLAYER_V6: Self = Self(6);

    pub const fn new(value: u8) -> Result<Self, InvalidScalar> {
        if value >= 1 && value <= 6 {
            Ok(Self(value))
        } else {
            Err(InvalidScalar)
        }
    }

    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MapId {
    FixedV1,
    GeneratedV1,
    GeneratedV2,
}
