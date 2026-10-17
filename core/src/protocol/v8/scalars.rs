use serde::{Serialize, Serializer};

use super::GAMEPLAY_PROTOCOL_VERSION;

pub(crate) use crate::protocol::v7::deserialize_bounded;

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
