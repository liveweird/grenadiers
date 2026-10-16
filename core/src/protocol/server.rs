use serde::Serialize;

use super::events::WireEvent;
use super::scalars::{Counter, GameplayVersion, MapId, RoomCode, RoomId, RulesVersion, Seat, Tick};
use super::snapshot::GameState;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum GameServerMessage {
    Authenticated {
        #[serde(rename = "protocolVersion")]
        protocol_version: GameplayVersion,
    },
    IncompatibleProtocol {
        #[serde(rename = "supportedVersion")]
        supported_version: GameplayVersion,
    },
    RoomState {
        room: RoomView,
        #[serde(rename = "youSeat")]
        you_seat: Seat,
    },
    Snapshot {
        room: RoomView,
        state: GameState,
        events: Vec<WireEvent>,
    },
    LeftRoom {
        #[serde(rename = "roomId")]
        room_id: RoomId,
        sequence: Counter,
    },
    Error {
        code: ErrorCode,
        sequence: Option<Counter>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomView {
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
    pub seats: [SeatStatus; 2],
    pub phase: RoomPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeatStatus {
    pub seat: Seat,
    pub occupied: bool,
    pub ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum RoomPhase {
    Waiting,
    Countdown {
        #[serde(rename = "endsAt")]
        ends_at: Tick,
    },
    Running,
    Result {
        #[serde(rename = "endedAt")]
        ended_at: Tick,
        result: WireRoundResult,
    },
    Interrupted {
        #[serde(rename = "endedAt")]
        ended_at: Tick,
        reason: InterruptReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum WireRoundResult {
    Winner { seat: Seat },
    Draw { reason: DrawReason },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DrawReason {
    MutualElimination,
    Timeout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum InterruptReason {
    ParticipantLeft,
    ConnectionLost,
    ServerOverloaded,
    ServerStopped,
    InternalError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorCode {
    InvalidMessage,
    AuthenticationFailed,
    ConnectionInUse,
    NotAuthenticated,
    AlreadyAuthenticated,
    AlreadyInRoom,
    NotInRoom,
    RoomNotFound,
    RoomFull,
    StaleRoom,
    InvalidPhase,
    StaleSequence,
    DuplicateAction,
    RateLimited,
    CapacityReached,
}
