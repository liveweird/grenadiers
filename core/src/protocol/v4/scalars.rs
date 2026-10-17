use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::GAMEPLAY_PROTOCOL_VERSION;

const MAX_ROUND_COUNT: u32 = 2_147_483_647;
const MAX_WINS: u8 = 3;

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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ResumeToken(String);

impl ResumeToken {
    pub fn new(value: impl Into<String>) -> Result<Self, super::super::InvalidScalar> {
        let value = value.into();
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value))
        } else {
            Err(super::super::InvalidScalar)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for ResumeToken {
    type Error = super::super::InvalidScalar;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ResumeToken> for String {
    fn from(value: ResumeToken) -> Self {
        value.0
    }
}

impl fmt::Debug for ResumeToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResumeToken([REDACTED])")
    }
}

macro_rules! bounded_scalar {
    ($name:ident, $storage:ty, $maximum:expr, $expectation:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
        #[serde(transparent)]
        pub struct $name($storage);
        impl $name {
            pub const fn new(value: $storage) -> Result<Self, super::super::InvalidScalar> {
                if value <= $maximum {
                    Ok(Self(value))
                } else {
                    Err(super::super::InvalidScalar)
                }
            }
            #[must_use]
            pub const fn get(self) -> $storage {
                self.0
            }
        }
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserialize_bounded(deserializer, u64::from($maximum), $expectation)
                    .map(|value| Self(value as $storage))
            }
        }
    };
}

bounded_scalar!(RoundCount, u32, MAX_ROUND_COUNT, "invalid round count");
bounded_scalar!(Wins, u8, MAX_WINS, "invalid wins");

fn deserialize_bounded<'de, D>(
    deserializer: D,
    maximum: u64,
    expectation: &'static str,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoundedVisitor {
        maximum: u64,
        expectation: &'static str,
    }
    impl Visitor<'_> for BoundedVisitor {
        type Value = u64;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.expectation)
        }
        fn visit_u64<E: de::Error>(self, value: u64) -> Result<u64, E> {
            self.validate(value)
        }
        fn visit_i64<E: de::Error>(self, value: i64) -> Result<u64, E> {
            u64::try_from(value)
                .map_err(|_| E::custom(self.expectation))
                .and_then(|value| self.validate(value))
        }
        fn visit_f64<E: de::Error>(self, value: f64) -> Result<u64, E> {
            if value.is_finite()
                && value.fract() == 0.0
                && value >= 0.0
                && value <= self.maximum as f64
            {
                Ok(value as u64)
            } else {
                Err(E::custom(self.expectation))
            }
        }
    }
    impl BoundedVisitor {
        fn validate<E: de::Error>(self, value: u64) -> Result<u64, E> {
            if value <= self.maximum {
                Ok(value)
            } else {
                Err(E::custom(self.expectation))
            }
        }
    }
    deserializer.deserialize_any(BoundedVisitor {
        maximum,
        expectation,
    })
}
