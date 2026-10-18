use crate::game::{DrawReason as DomainDrawReason, RoundResult, SeatId, SpeedLevel};
use crate::protocol::{Counter, RoomCode, RoomId, Seat, Tick, WireRoundResult};
use crate::room::reconnect::ReservationState;
use crate::room::series::{
    CompletedRound as DomainCompletedRound, MatchResult as DomainMatchResult,
};
use crate::room::{
    CountdownKind as DomainCountdownKind, InterruptReason, MatchLifecycle,
    MatchPhase as DomainMatchPhase, RoomSettings,
};

use super::projection_binding::{available_items, validate_expanded_room};
use super::{
    CompletedRound, ConnectionStatus, CountdownKind, GameSettings, MapId, MatchInterruptReason,
    MatchResult, MatchWinnerReason, ProjectionError, RoomPhase, RoomView, RoundCount, RulesVersion,
    Scores, SeatStatus, Wins,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomMetadata {
    pub settings: GameSettings,
    pub settings_revision: Counter,
    pub settings_editor: Option<Seat>,
    pub room_id: RoomId,
    pub room_code: RoomCode,
    pub revision: Counter,
    pub rules_version: RulesVersion,
    pub map_id: MapId,
}

pub fn expanded_settings(settings: GameSettings) -> Result<RoomSettings, ProjectionError> {
    let speed = match settings.starting_speed_level() {
        0 => SpeedLevel::Base,
        1 => SpeedLevel::One,
        2 => SpeedLevel::Two,
        3 => SpeedLevel::Three,
        4 => SpeedLevel::Four,
        _ => return Err(ProjectionError::InvalidRules),
    };
    let enabled = crate::game::EnabledItems::from_bits(settings.enabled_items().mask().bits())
        .ok_or(ProjectionError::InvalidItems)?;
    RoomSettings::new(
        speed,
        settings.turn_assist_units(),
        settings.wins_to_finish(),
        u64::from(settings.bomb_fuse_ticks()),
    )
    .map(|value| value.with_enabled_items(enabled))
    .map_err(|_| ProjectionError::InvalidRules)
}

pub fn project_expanded_room(
    metadata: RoomMetadata,
    lifecycle: &MatchLifecycle,
) -> Result<RoomView, ProjectionError> {
    if expanded_settings(metadata.settings)? != lifecycle.settings() {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let series = lifecycle.series();
    let scores = series.scores();
    let votes = series.rematch_votes();
    let room = RoomView {
        settings: metadata.settings,
        settings_revision: metadata.settings_revision,
        settings_editor: metadata.settings_editor,
        available_items: available_items(metadata.rules_version),
        room_id: metadata.room_id,
        room_code: metadata.room_code,
        round_id: counter(lifecycle.round_attempt())?,
        revision: metadata.revision,
        tick: tick(lifecycle.tick().0)?,
        rules_version: metadata.rules_version,
        map_id: metadata.map_id,
        round_number: RoundCount::new(series.actual_round_number())
            .map_err(|_| ProjectionError::InvalidClock)?,
        scores: Scores {
            one: wins(scores[0])?,
            two: wins(scores[1])?,
        },
        last_round: series
            .latest_completed_round()
            .map(completed_round)
            .transpose()?,
        seats: [
            seat(lifecycle, votes, 0, Seat::One)?,
            seat(lifecycle, votes, 1, Seat::Two)?,
        ],
        phase: phase(lifecycle.phase())?,
    };
    validate_expanded_room(&room)?;
    Ok(room)
}

fn completed_round(value: DomainCompletedRound) -> Result<CompletedRound, ProjectionError> {
    Ok(CompletedRound {
        round_id: counter(value.epoch())?,
        round_number: counter(u64::from(value.assignment().round_number().get()))?,
        ended_at: tick(value.ended_at().0)?,
        result: round_result(value.result()),
    })
}

fn phase(value: DomainMatchPhase) -> Result<RoomPhase, ProjectionError> {
    Ok(match value {
        DomainMatchPhase::Waiting => RoomPhase::Waiting,
        DomainMatchPhase::Countdown { kind, ends_at } => RoomPhase::Countdown {
            kind: match kind {
                DomainCountdownKind::Initial => CountdownKind::Initial,
                DomainCountdownKind::InterRound => CountdownKind::InterRound,
            },
            ends_at: tick(ends_at.0)?,
        },
        DomainMatchPhase::Running => RoomPhase::Running,
        DomainMatchPhase::Result { next_countdown_at } => RoomPhase::Result {
            hold_until: tick(next_countdown_at.0)?,
        },
        DomainMatchPhase::MatchResult {
            outcome,
            rematch_at,
        } => RoomPhase::MatchResult {
            ended_at: tick(outcome.ended_at().0)?,
            result: match_result(outcome.result())?,
            rematch_at: tick(rematch_at.0)?,
        },
        DomainMatchPhase::Interrupted { ended_at, reason } => RoomPhase::Interrupted {
            ended_at: tick(ended_at.0)?,
            reason: interrupt(reason)?,
        },
    })
}

fn match_result(value: DomainMatchResult) -> Result<MatchResult, ProjectionError> {
    Ok(match value {
        DomainMatchResult::ScoreWinner(seat) => MatchResult::Winner {
            seat: wire_seat(seat),
            reason: MatchWinnerReason::RoundWins,
        },
        DomainMatchResult::ForfeitWinner(seat) => MatchResult::Winner {
            seat: wire_seat(seat),
            reason: MatchWinnerReason::Forfeit,
        },
        DomainMatchResult::Abandoned => MatchResult::Abandoned,
        DomainMatchResult::Interrupted(reason) => MatchResult::Interrupted {
            reason: interrupt(reason)?,
        },
    })
}

fn interrupt(value: InterruptReason) -> Result<MatchInterruptReason, ProjectionError> {
    Ok(match value {
        InterruptReason::ParticipantLeft => return Err(ProjectionError::InvalidResult),
        InterruptReason::ConnectionLost => MatchInterruptReason::ConnectionLost,
        InterruptReason::ServerOverloaded => MatchInterruptReason::ServerOverloaded,
        InterruptReason::ServerStopped => MatchInterruptReason::ServerStopped,
        InterruptReason::InternalError => MatchInterruptReason::InternalError,
    })
}

fn seat(
    lifecycle: &MatchLifecycle,
    votes: [bool; 2],
    index: usize,
    wire: Seat,
) -> Result<SeatStatus, ProjectionError> {
    let domain = lifecycle.seats()[index];
    let connection = match lifecycle.reconnect_grace().seat(SeatId::ALL[index]) {
        ReservationState::Vacant => ConnectionStatus::Vacant,
        ReservationState::Connected => ConnectionStatus::Connected,
        ReservationState::Disconnected { expires_at } => ConnectionStatus::Disconnected {
            expires_at: tick(expires_at.0)?,
        },
        ReservationState::Expired => ConnectionStatus::Expired,
    };
    Ok(SeatStatus {
        seat: wire,
        occupied: domain.occupied(),
        ready: domain.ready(),
        rematch_ready: votes[index],
        connection,
    })
}

fn round_result(value: RoundResult) -> WireRoundResult {
    match value {
        RoundResult::Winner(seat) => WireRoundResult::Winner {
            seat: wire_seat(seat),
        },
        RoundResult::Draw(DomainDrawReason::MutualElimination) => WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::MutualElimination,
        },
        RoundResult::Draw(DomainDrawReason::Timeout) => WireRoundResult::Draw {
            reason: crate::protocol::DrawReason::Timeout,
        },
    }
}

fn wire_seat(value: SeatId) -> Seat {
    match value {
        SeatId::One => Seat::One,
        SeatId::Two => Seat::Two,
    }
}

fn counter(value: u64) -> Result<Counter, ProjectionError> {
    u32::try_from(value)
        .ok()
        .and_then(|value| Counter::new(value).ok())
        .ok_or(ProjectionError::InvalidClock)
}

fn tick(value: u64) -> Result<Tick, ProjectionError> {
    Tick::new(value).map_err(|_| ProjectionError::InvalidClock)
}

fn wins(value: u8) -> Result<Wins, ProjectionError> {
    Wins::new(value).map_err(|_| ProjectionError::InvalidResult)
}
