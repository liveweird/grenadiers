use crate::game::{DrawReason as DomainDrawReason, RoundResult, SeatId};
use crate::room::series::{
    CompletedRound as DomainCompletedRound, MatchResult as DomainMatchResult,
};
use crate::room::{
    CountdownKind as DomainCountdownKind, InterruptReason, MatchLifecycle,
    MatchPhase as DomainMatchPhase,
};

use super::super::{
    Counter, DrawReason, MapId, RoomCode, RoomId, RulesVersion, Seat, Tick, WireRoundResult,
};
use super::projection_validation::validate_room_view;
use super::{
    CompletedRound, CountdownKind, MatchInterruptReason, MatchResult, MatchWinnerReason, RoomPhase,
    RoomView, RoundCount, Scores, SeatStatus, Wins,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomMetadata {
    pub room_id: RoomId,
    pub room_code: RoomCode,
    pub revision: Counter,
    pub rules_version: RulesVersion,
    pub map_id: MapId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomProjectionError {
    UnsupportedRulesVersion,
    InvalidCounter,
    InvalidTick,
    InvalidScores,
    InvalidSeats,
    InvalidCompletedRound,
    InvalidPhase,
    InvalidMatchResult,
}

impl std::fmt::Display for RoomProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for RoomProjectionError {}

pub fn project_room_view(
    metadata: RoomMetadata,
    lifecycle: &MatchLifecycle,
) -> Result<RoomView, RoomProjectionError> {
    let series = lifecycle.series();
    let scores = series.scores();
    let votes = series.rematch_votes();
    let room = RoomView {
        room_id: metadata.room_id,
        room_code: metadata.room_code,
        round_id: wire_counter(lifecycle.round_attempt())?,
        revision: metadata.revision,
        tick: wire_tick(lifecycle.tick().0)?,
        rules_version: metadata.rules_version,
        map_id: metadata.map_id,
        round_number: RoundCount::new(series.actual_round_number())
            .map_err(|_| RoomProjectionError::InvalidCounter)?,
        scores: Scores {
            one: wire_wins(scores[0])?,
            two: wire_wins(scores[1])?,
        },
        last_round: series
            .latest_completed_round()
            .map(project_completed_round)
            .transpose()?,
        seats: [
            SeatStatus {
                seat: Seat::One,
                occupied: lifecycle.seats()[0].occupied(),
                ready: lifecycle.seats()[0].ready(),
                rematch_ready: votes[0],
            },
            SeatStatus {
                seat: Seat::Two,
                occupied: lifecycle.seats()[1].occupied(),
                ready: lifecycle.seats()[1].ready(),
                rematch_ready: votes[1],
            },
        ],
        phase: project_phase(lifecycle.phase())?,
    };
    validate_room_view(&room)?;
    Ok(room)
}

fn project_completed_round(
    completed: DomainCompletedRound,
) -> Result<CompletedRound, RoomProjectionError> {
    Ok(CompletedRound {
        round_id: wire_counter(completed.epoch())?,
        round_number: wire_counter(u64::from(completed.assignment().round_number().get()))?,
        ended_at: wire_tick(completed.ended_at().0)?,
        result: wire_round_result(completed.result())?,
    })
}

fn project_phase(phase: DomainMatchPhase) -> Result<RoomPhase, RoomProjectionError> {
    Ok(match phase {
        DomainMatchPhase::Waiting => RoomPhase::Waiting,
        DomainMatchPhase::Countdown { kind, ends_at } => RoomPhase::Countdown {
            kind: match kind {
                DomainCountdownKind::Initial => CountdownKind::Initial,
                DomainCountdownKind::InterRound => CountdownKind::InterRound,
            },
            ends_at: wire_tick(ends_at.0)?,
        },
        DomainMatchPhase::Running => RoomPhase::Running,
        DomainMatchPhase::Result { next_countdown_at } => RoomPhase::Result {
            next_countdown_at: wire_tick(next_countdown_at.0)?,
        },
        DomainMatchPhase::MatchResult {
            outcome,
            rematch_at,
        } => RoomPhase::MatchResult {
            ended_at: wire_tick(outcome.ended_at().0)?,
            result: wire_match_result(outcome.result())?,
            rematch_at: wire_tick(rematch_at.0)?,
        },
        DomainMatchPhase::Interrupted { ended_at, reason } => RoomPhase::Interrupted {
            ended_at: wire_tick(ended_at.0)?,
            reason: wire_interrupt(reason)?,
        },
    })
}

fn wire_match_result(result: DomainMatchResult) -> Result<MatchResult, RoomProjectionError> {
    Ok(match result {
        DomainMatchResult::ScoreWinner(seat) => MatchResult::Winner {
            seat: wire_seat(seat)?,
            reason: MatchWinnerReason::RoundWins,
        },
        DomainMatchResult::ForfeitWinner(seat) => MatchResult::Winner {
            seat: wire_seat(seat)?,
            reason: MatchWinnerReason::Forfeit,
        },
        DomainMatchResult::Abandoned => MatchResult::Abandoned,
        DomainMatchResult::Interrupted(reason) => MatchResult::Interrupted {
            reason: wire_interrupt(reason)?,
        },
    })
}

fn wire_interrupt(reason: InterruptReason) -> Result<MatchInterruptReason, RoomProjectionError> {
    match reason {
        InterruptReason::ParticipantLeft => Err(RoomProjectionError::InvalidMatchResult),
        InterruptReason::ConnectionLost => Ok(MatchInterruptReason::ConnectionLost),
        InterruptReason::ServerOverloaded => Ok(MatchInterruptReason::ServerOverloaded),
        InterruptReason::ServerStopped => Ok(MatchInterruptReason::ServerStopped),
        InterruptReason::InternalError => Ok(MatchInterruptReason::InternalError),
    }
}

fn wire_round_result(result: RoundResult) -> Result<WireRoundResult, RoomProjectionError> {
    Ok(match result {
        RoundResult::Winner(seat) => WireRoundResult::Winner {
            seat: wire_seat(seat)?,
        },
        RoundResult::Draw(DomainDrawReason::MutualElimination) => WireRoundResult::Draw {
            reason: DrawReason::MutualElimination,
        },
        RoundResult::Draw(DomainDrawReason::Timeout) => WireRoundResult::Draw {
            reason: DrawReason::Timeout,
        },
    })
}

fn wire_seat(seat: SeatId) -> Result<Seat, RoomProjectionError> {
    Seat::try_from(seat).map_err(|_| RoomProjectionError::InvalidMatchResult)
}

fn wire_counter(value: u64) -> Result<Counter, RoomProjectionError> {
    u32::try_from(value)
        .ok()
        .and_then(|value| Counter::new(value).ok())
        .ok_or(RoomProjectionError::InvalidCounter)
}

fn wire_tick(value: u64) -> Result<Tick, RoomProjectionError> {
    Tick::new(value).map_err(|_| RoomProjectionError::InvalidTick)
}

fn wire_wins(value: u8) -> Result<Wins, RoomProjectionError> {
    Wins::new(value).map_err(|_| RoomProjectionError::InvalidScores)
}

#[cfg(test)]
#[path = "projection_test_support.rs"]
mod test_support;

#[cfg(test)]
#[path = "projection_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "projection_validation_tests.rs"]
mod validation_tests;

#[cfg(test)]
#[path = "projection_terminal_validation_tests.rs"]
mod terminal_validation_tests;
