use serde::{Deserialize, Serialize};

use super::GAMEPLAY_PROTOCOL_VERSION;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Seat {
    One,
    Two,
    Three,
    Four,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameplayVersion;

impl Serialize for GameplayVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(GAMEPLAY_PROTOCOL_VERSION)
    }
}
