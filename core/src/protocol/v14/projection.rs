use std::collections::BTreeSet;

use super::{ConnectionStatus, Controller, MatchResult, RoomPhase, RoomView, RoundResult, Seat};
use crate::room::SeatControl;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidRules,
    InvalidClock,
    InvalidArena,
    InvalidActors,
    InvalidBombs,
    InvalidFlames,
    InvalidPickups,
    InvalidResult,
    InvalidEffects,
    InvalidItems,
    InvalidReports,
    InvalidEvent,
    InvalidExpandedBinding,
    EncodingFailed,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProjectionError {}

impl Seat {
    pub(crate) const fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }
}

fn valid_result(result: RoundResult, seats: &BTreeSet<usize>) -> bool {
    match result {
        RoundResult::Winner { seat } => seats.contains(&seat.index()),
        RoundResult::Draw { .. } => true,
    }
}

pub fn validate_room_view(room: &RoomView) -> Result<(), ProjectionError> {
    let configured = room.configuration.domain().controllers();
    let count = configured.len();
    if room.seats.len() != count || room.scores.len() != count || !(2..=4).contains(&count) {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    if room.rules_version.get() < 6 && count != 2 {
        return Err(ProjectionError::InvalidRules);
    }
    if room.settings_editor.is_some_and(|seat| seat != Seat::One) {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let expected: BTreeSet<_> = (0..count).collect();
    let status_seats: BTreeSet<_> = room
        .seats
        .iter()
        .map(|status| status.seat.index())
        .collect();
    let score_seats: BTreeSet<_> = room.scores.iter().map(|score| score.seat.index()).collect();
    if status_seats != expected || score_seats != expected {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    for status in &room.seats {
        let expected_control = configured[status.seat.index()];
        let valid = match (expected_control, status.controller, status.connection) {
            (SeatControl::Human, Controller::Human, ConnectionStatus::ServerControlled) => false,
            (SeatControl::Human, Controller::Human, _) => true,
            (
                SeatControl::Cpu,
                Controller::Cpu { difficulty },
                ConnectionStatus::ServerControlled,
            ) => {
                status.occupied
                    && room
                        .configuration
                        .domain()
                        .cpu_difficulty()
                        .is_some_and(|value| {
                            matches!(
                                (value, difficulty),
                                (crate::room::CpuDifficulty::Easy, super::Difficulty::Easy)
                                    | (
                                        crate::room::CpuDifficulty::Standard,
                                        super::Difficulty::Standard
                                    )
                            )
                        })
            }
            _ => false,
        };
        if !valid {
            return Err(ProjectionError::InvalidExpandedBinding);
        }
    }
    if room
        .scores
        .iter()
        .any(|score| score.wins.get() > room.settings.wins_to_finish())
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    if room
        .last_round
        .is_some_and(|round| !valid_result(round.result, &expected))
    {
        return Err(ProjectionError::InvalidResult);
    }
    if let RoomPhase::MatchResult {
        result: MatchResult::Winner { seat, .. },
        ..
    } = room.phase
        && !expected.contains(&seat.index())
    {
        return Err(ProjectionError::InvalidResult);
    }
    Ok(())
}
