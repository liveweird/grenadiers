use serde::{Deserialize, Deserializer};

use super::super::{Counter, Direction, ProtocolVersionInput, RoomCode, RoomId, SessionToken};
use super::ResumeToken;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum GameClientMessage {
    Authenticate {
        #[serde(rename = "protocolVersion")]
        protocol_version: ProtocolVersionInput,
        token: SessionToken,
    },
    CreateRoom {
        sequence: Counter,
    },
    JoinRoom {
        sequence: Counter,
        #[serde(rename = "roomCode")]
        room_code: RoomCode,
    },
    ResumeRoom {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "resumeToken")]
        resume_token: ResumeToken,
    },
    Ready {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
        ready: bool,
    },
    Movement {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
        #[serde(deserialize_with = "required_direction")]
        direction: Option<Direction>,
    },
    PlaceBomb {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
        #[serde(rename = "actionId")]
        action_id: Counter,
    },
    LeaveRoom {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
    },
    Rematch {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
        ready: bool,
    },
}

fn required_direction<'de, D>(deserializer: D) -> Result<Option<Direction>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Direction>::deserialize(deserializer)
}
