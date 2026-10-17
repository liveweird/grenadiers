use serde::Serialize;

use super::super::{
    Counter, ErrorCode, GameState, MapId, ProtocolVersionInput, RoomCode, RoomId, RulesVersion,
    Seat, Tick, WireEvent, WireRoundResult,
};
use super::{GameplayVersion, RoundCount, Wins};

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
        #[serde(rename = "nextCountdownAt")]
        next_countdown_at: Tick,
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
