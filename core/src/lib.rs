#![forbid(unsafe_code)]

mod item_spec;

pub mod game;
pub mod protocol;
pub mod room;

use serde::{Deserialize, Deserializer, Serialize, de::Error as _};

/// Protocol version understood by this server.
pub const PROTOCOL_VERSION: u16 = 1;

/// Initial game capacity. This does not limit transport connections.
pub const MAX_PLAYERS: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ClientMessage {
    Ping {
        #[serde(deserialize_with = "deserialize_sequence")]
        sequence: u32,
    },
}

fn deserialize_sequence<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let sequence = u32::deserialize(deserializer)?;
    if sequence <= i32::MAX.cast_unsigned() {
        Ok(sequence)
    } else {
        Err(D::Error::custom("sequence exceeds 2147483647"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ServerMessage {
    Welcome {
        #[serde(rename = "protocolVersion")]
        protocol_version: u16,
        #[serde(rename = "maxPlayers")]
        max_players: u8,
    },
    Pong {
        sequence: u32,
    },
    Error {
        code: ErrorCode,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    InvalidMessage,
}

impl ServerMessage {
    #[must_use]
    pub const fn welcome() -> Self {
        Self::Welcome {
            protocol_version: PROTOCOL_VERSION,
            max_players: MAX_PLAYERS,
        }
    }

    #[must_use]
    pub fn invalid_message() -> Self {
        Self::Error {
            code: ErrorCode::InvalidMessage,
            message: "Invalid client message".to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_protocol_messages() {
        assert_eq!(
            serde_json::to_value(ServerMessage::welcome()).unwrap(),
            serde_json::json!({"type": "welcome", "protocolVersion": 1, "maxPlayers": 2})
        );
        assert_eq!(
            serde_json::to_value(ServerMessage::Pong { sequence: 7 }).unwrap(),
            serde_json::json!({"type": "pong", "sequence": 7})
        );
        assert_eq!(
            serde_json::to_value(ServerMessage::invalid_message()).unwrap(),
            serde_json::json!({
                "type": "error",
                "code": "invalidMessage",
                "message": "Invalid client message"
            })
        );
    }

    #[test]
    fn accepts_only_bounded_strict_ping_messages() {
        assert_eq!(
            serde_json::from_str::<ClientMessage>(r#"{"type":"ping","sequence":2147483647}"#)
                .unwrap(),
            ClientMessage::Ping {
                sequence: 2_147_483_647
            }
        );
        assert!(
            serde_json::from_str::<ClientMessage>(r#"{"type":"ping","sequence":1,"extra":1}"#)
                .is_err()
        );
        assert!(
            serde_json::from_str::<ClientMessage>(r#"{"type":"ping","sequence":2147483648}"#)
                .is_err()
        );
    }
}
