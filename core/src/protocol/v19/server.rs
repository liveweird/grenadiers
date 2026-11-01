use super::{GameState, ResumeToken, RoomConfiguration, RoomSettings, Seat, SnapshotEvents};
use crate::protocol::{Counter, ProtocolVersionInput, RoomCode, RoomId, Tick};
use serde::Serialize;

pub use crate::protocol::v14::{
    CompletedRound, ConnectionStatus, CountdownKind, DrawReason, ErrorCode, MatchInterruptReason,
    MatchResult, MatchWinnerReason, RoundResult, Score, Scores,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Controller {
    Human,
    Cpu { difficulty: super::Difficulty },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeatStatus {
    pub seat: Seat,
    pub occupied: bool,
    pub ready: bool,
    pub rematch_ready: bool,
    pub controller: Controller,
    pub connection: ConnectionStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct RulesVersion(u8);
impl RulesVersion {
    pub const ARENA_V10: Self = Self(10);
    pub const fn new(value: u8) -> Result<Self, crate::protocol::InvalidScalar> {
        if value == 10 {
            Ok(Self(value))
        } else {
            Err(crate::protocol::InvalidScalar)
        }
    }
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameplayVersion;
impl Serialize for GameplayVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(super::GAMEPLAY_PROTOCOL_VERSION)
    }
}

pub use crate::protocol::v16::RoomPhase;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomView {
    pub settings: RoomSettings,
    pub settings_revision: Counter,
    pub settings_editor: Option<Seat>,
    pub available_items: super::ItemSet,
    pub configuration: RoomConfiguration,
    #[serde(rename = "roomId")]
    pub room_id: RoomId,
    #[serde(rename = "roomCode")]
    pub room_code: RoomCode,
    #[serde(rename = "roundId")]
    pub round_id: Counter,
    pub revision: Counter,
    pub tick: Tick,
    #[serde(rename = "rulesVersion")]
    pub rules_version: RulesVersion,
    #[serde(rename = "mapId")]
    pub map_id: super::MapId,
    #[serde(rename = "roundNumber")]
    pub round_number: super::RoundCount,
    pub scores: Scores,
    #[serde(rename = "lastRound")]
    pub last_round: Option<CompletedRound>,
    pub seats: Vec<SeatStatus>,
    pub phase: RoomPhase,
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
        code: ErrorCode,
        sequence: Option<Counter>,
    },
}
