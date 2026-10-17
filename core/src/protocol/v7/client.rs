use serde::{Deserialize, Deserializer};

use super::super::{Counter, Direction, ProtocolVersionInput, RoomCode, RoomId, SessionToken};
use super::{GameSettings, ResumeToken};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ManualActionKind {
    Bomb,
    Equipment,
}

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
        #[serde(rename = "settingsRevision")]
        settings_revision: Counter,
    },
    UpdateSettings {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
        #[serde(rename = "settingsRevision")]
        settings_revision: Counter,
        settings: GameSettings,
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
    ManualAction {
        sequence: Counter,
        #[serde(rename = "roomId")]
        room_id: RoomId,
        #[serde(rename = "roundId")]
        round_id: Counter,
        #[serde(rename = "actionId")]
        action_id: Counter,
        action: ManualActionKind,
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

fn required_direction<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Direction>, D::Error> {
    Option::deserialize(d)
}
