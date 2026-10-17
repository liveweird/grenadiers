use serde::Serialize;

use super::super::{
    Counter, GameState, MapId, ProtocolVersionInput, RoomCode, RoomId, RulesVersion, Seat, Tick,
    WireEvent, WireRoundResult,
};
use super::{GameplayVersion, ResumeToken, RoundCount, Wins};

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
        #[serde(rename = "lastBombActionId")]
        last_bomb_action_id: Option<Counter>,
        state: Option<GameState>,
    },
    RoomState {
        room: RoomView,
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
    SeatReplaced {
        #[serde(rename = "roomId")]
        room_id: RoomId,
    },
    Error {
        code: V4ErrorCode,
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
    #[serde(rename = "roundNumber")]
    pub round_number: RoundCount,
    pub scores: Scores,
    #[serde(rename = "lastRound")]
    pub last_round: Option<CompletedRound>,
    pub seats: [SeatStatus; 2],
    pub phase: RoomPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Scores {
    pub one: Wins,
    pub two: Wins,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeatStatus {
    pub seat: Seat,
    pub occupied: bool,
    pub ready: bool,
    pub rematch_ready: bool,
    pub connection: ConnectionStatus,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletedRound {
    pub round_id: Counter,
    pub round_number: Counter,
    pub ended_at: Tick,
    pub result: WireRoundResult,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CountdownKind {
    Initial,
    InterRound,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum RoomPhase {
    Waiting,
    Countdown {
        kind: CountdownKind,
        #[serde(rename = "endsAt")]
        ends_at: Tick,
    },
    Running,
    Result {
        #[serde(rename = "holdUntil")]
        hold_until: Tick,
    },
    MatchResult {
        #[serde(rename = "endedAt")]
        ended_at: Tick,
        result: MatchResult,
        #[serde(rename = "rematchAt")]
        rematch_at: Tick,
    },
    Interrupted {
        #[serde(rename = "endedAt")]
        ended_at: Tick,
        reason: MatchInterruptReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum MatchResult {
    Winner {
        seat: Seat,
        reason: MatchWinnerReason,
    },
    Abandoned,
    Interrupted {
        reason: MatchInterruptReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchWinnerReason {
    RoundWins,
    Forfeit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MatchInterruptReason {
    ConnectionLost,
    ServerOverloaded,
    ServerStopped,
    InternalError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum V4ErrorCode {
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
    ResumeRejected,
}
