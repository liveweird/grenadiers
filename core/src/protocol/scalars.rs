use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 2;
pub const MAX_COUNTER: u32 = 2_147_483_647;
pub const MAX_SAFE_TICK: u64 = 9_007_199_254_740_991;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidScalar;

impl fmt::Display for InvalidScalar {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid protocol scalar")
    }
}

impl std::error::Error for InvalidScalar {}

macro_rules! validated_string {
    ($name:ident, $length:expr, $valid:expr) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: &str) -> Result<Self, InvalidScalar> {
                if value.len() == $length && value.bytes().all($valid) {
                    Ok(Self(value.to_owned()))
                } else {
                    Err(InvalidScalar)
                }
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(&value).map_err(|_| de::Error::custom("invalid canonical string"))
            }
        }

        impl TryFrom<&str> for $name {
            type Error = InvalidScalar;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }
    };
}

validated_string!(SessionToken, 64, |byte: u8| byte.is_ascii_digit()
    || (b'a'..=b'f').contains(&byte));
validated_string!(RoomId, 32, |byte: u8| byte.is_ascii_digit()
    || (b'a'..=b'f').contains(&byte));
validated_string!(RoomCode, 8, |byte: u8| (b'A'..=b'H').contains(&byte)
    || (b'J'..=b'N').contains(&byte)
    || (b'P'..=b'Z').contains(&byte)
    || (b'2'..=b'9').contains(&byte));

impl fmt::Debug for SessionToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SessionToken([REDACTED])")
    }
}

impl fmt::Debug for RoomId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("RoomId").field(&self.0).finish()
    }
}

impl fmt::Debug for RoomCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_tuple("RoomCode").field(&self.0).finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ProtocolVersionInput(u16);

impl ProtocolVersionInput {
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl<'de> Deserialize<'de> for ProtocolVersionInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bounded(
            deserializer,
            0,
            u64::from(u16::MAX),
            "invalid protocol version",
        )
        .map(|value| Self(value as u16))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Counter(u32);

impl Counter {
    pub const fn new(value: u32) -> Result<Self, InvalidScalar> {
        if value >= 1 && value <= MAX_COUNTER {
            Ok(Self(value))
        } else {
            Err(InvalidScalar)
        }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Counter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_bounded(deserializer, 1, u64::from(MAX_COUNTER), "invalid counter")
            .map(|value| Self(value as u32))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Tick(u64);

impl Tick {
    pub const fn new(value: u64) -> Result<Self, InvalidScalar> {
        if value <= MAX_SAFE_TICK {
            Ok(Self(value))
        } else {
            Err(InvalidScalar)
        }
    }

    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl TryFrom<u64> for Tick {
    type Error = InvalidScalar;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct EntityId(u32);

impl EntityId {
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Cell {
    column: u8,
    row: u8,
}

impl Cell {
    pub const fn new(column: u8, row: u8) -> Result<Self, InvalidScalar> {
        if column <= 14 && row <= 12 {
            Ok(Self { column, row })
        } else {
            Err(InvalidScalar)
        }
    }

    #[must_use]
    pub const fn column(self) -> u8 {
        self.column
    }

    #[must_use]
    pub const fn row(self) -> u8 {
        self.row
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Position {
    x: u16,
    y: u16,
}

impl Position {
    pub const fn new(x: u16, y: u16) -> Result<Self, InvalidScalar> {
        if x <= 15_359 && y <= 13_311 {
            Ok(Self { x, y })
        } else {
            Err(InvalidScalar)
        }
    }

    #[must_use]
    pub const fn x(self) -> u16 {
        self.x
    }

    #[must_use]
    pub const fn y(self) -> u16 {
        self.y
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Seat {
    One,
    Two,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    North,
    East,
    South,
    West,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GameplayVersion;

impl Serialize for GameplayVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u16(GAMEPLAY_PROTOCOL_VERSION)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RulesVersion {
    #[default]
    ClassicV1,
    ClassicV2,
    ClassicV3,
}

impl RulesVersion {
    #[must_use]
    pub const fn get(self) -> u16 {
        match self {
            Self::ClassicV1 => 1,
            Self::ClassicV2 => 2,
            Self::ClassicV3 => 3,
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
}

fn deserialize_bounded<'de, D>(
    deserializer: D,
    minimum: u64,
    maximum: u64,
    expectation: &'static str,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedVisitor {
        minimum: u64,
        maximum: u64,
        expectation: &'static str,
    }

    impl Visitor<'_> for BoundedVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.expectation)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.validate(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            u64::try_from(value)
                .map_err(|_| E::custom(self.expectation))
                .and_then(|value| self.validate(value))
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if value.is_finite()
                && value.fract() == 0.0
                && value >= self.minimum as f64
                && value <= self.maximum as f64
            {
                Ok(value as u64)
            } else {
                Err(E::custom(self.expectation))
            }
        }
    }

    impl BoundedVisitor {
        fn validate<E>(self, value: u64) -> Result<u64, E>
        where
            E: de::Error,
        {
            if (self.minimum..=self.maximum).contains(&value) {
                Ok(value)
            } else {
                Err(E::custom(self.expectation))
            }
        }
    }

    deserializer.deserialize_any(BoundedVisitor {
        minimum,
        maximum,
        expectation,
    })
}
