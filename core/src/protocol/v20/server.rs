use super::{GameState, ResumeToken, Seat, SnapshotEvents};
use crate::protocol::{Counter, ProtocolVersionInput, RoomId};
use serde::Serialize;

pub type RoomView = crate::protocol::v19::RoomView;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameplayVersion;

impl Serialize for GameplayVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(super::GAMEPLAY_PROTOCOL_VERSION)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum GameServerMessage {
    Authenticated {
        #[serde(rename = "protocolVersion")]
        protocol_version: GameplayVersion,
    },
    IncompatibleProtocol {
        #[serde(rename = "supportedVersion")]
        supported_version: ProtocolVersionInput,
    },
    Admitted {
        sequence: Counter,
        room: RoomView,
        #[serde(rename = "youSeat")]
        you_seat: Seat,
        #[serde(rename = "resumeToken")]
        resume_token: ResumeToken,
        #[serde(rename = "lastActionId")]
        last_action_id: Option<Counter>,
        state: Option<GameState>,
    },
    RoomState {
        room: RoomView,
    },
    Snapshot {
        room: RoomView,
        state: GameState,
        events: SnapshotEvents,
    },
    LeftRoom {
        #[serde(rename = "roomId")]
        room_id: RoomId,
        sequence: Counter,
    },
    SeatReplaced {
        #[serde(rename = "roomId")]
        room_id: RoomId,
    },
    Error {
        code: super::ErrorCode,
        sequence: Option<Counter>,
    },
}
