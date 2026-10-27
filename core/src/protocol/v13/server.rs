use serde::{Deserialize, Serialize};

use super::super::{Counter, ProtocolVersionInput, RoomCode, RoomId, Seat, Tick};
use super::{
    CompletedRound, GameSettings, GameState, GameplayVersion, ItemSet, MapId, ResumeToken,
    RoomPhase, RoundCount, RulesVersion, Scores, SnapshotEvents, V11ErrorCode,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Difficulty {
    Easy,
    Standard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum RoomMode {
    HumanDuel,
    Solo { difficulty: Difficulty },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Controller {
    Human,
    Cpu { difficulty: Difficulty },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ConnectionStatus {
    Vacant,
    Connected,
    Disconnected {
        #[serde(rename = "expiresAt")]
        expires_at: Tick,
    },
    Expired,
    ServerControlled,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomView {
    pub settings: GameSettings,
    pub settings_revision: Counter,
    pub settings_editor: Option<Seat>,
    pub available_items: ItemSet,
    pub mode: RoomMode,
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
    pub map_id: MapId,
    #[serde(rename = "roundNumber")]
    pub round_number: RoundCount,
    pub scores: Scores,
    #[serde(rename = "lastRound")]
    pub last_round: Option<CompletedRound>,
    pub seats: [SeatStatus; 2],
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
        code: V11ErrorCode,
        sequence: Option<Counter>,
    },
}
