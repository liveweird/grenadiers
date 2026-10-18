use crate::game::SpeedLevel;
use crate::protocol::{Counter, MapId, RoomCode, RoomId, RulesVersion, Seat};
use crate::room::{MatchLifecycle, RoomSettings};

use super::{
    CompletedRound, ConnectionStatus, CountdownKind, GameSettings, ItemSet, MatchInterruptReason,
    MatchResult, MatchWinnerReason, ProjectionError, RoomPhase, RoomView, RoundCount, Scores,
    SeatStatus, Wins,
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

pub fn classic_settings(settings: GameSettings) -> Result<RoomSettings, ProjectionError> {
    if settings.enabled_items() != ItemSet::empty() {
        return Err(ProjectionError::InvalidItems);
    }
    let speed = match settings.starting_speed_level() {
        0 => SpeedLevel::Base,
        1 => SpeedLevel::One,
        2 => SpeedLevel::Two,
        3 => SpeedLevel::Three,
        4 => SpeedLevel::Four,
        _ => return Err(ProjectionError::InvalidRules),
    };
    RoomSettings::new(
        speed,
        settings.turn_assist_units(),
        settings.wins_to_finish(),
        u64::from(settings.bomb_fuse_ticks()),
    )
    .map_err(|_| ProjectionError::InvalidRules)
}

pub fn project_classic_room(
    metadata: RoomMetadata,
    lifecycle: &MatchLifecycle,
) -> Result<RoomView, ProjectionError> {
    let settings = classic_settings(metadata.settings)?;
    let legacy = crate::protocol::v6::project_room_view(
        crate::protocol::v6::RoomMetadata {
            settings: crate::protocol::v6::GameSettings::from_domain(settings)
                .map_err(|_| ProjectionError::InvalidItems)?,
            settings_revision: metadata.settings_revision,
            settings_editor: metadata.settings_editor,
            room_id: metadata.room_id.clone(),
            room_code: metadata.room_code.clone(),
            revision: metadata.revision,
            rules_version: metadata.rules_version,
            map_id: metadata.map_id,
        },
        lifecycle,
    )
    .map_err(|_| ProjectionError::InvalidClassicBinding)?;
    Ok(RoomView {
        settings: metadata.settings,
        settings_revision: legacy.settings_revision,
        settings_editor: legacy.settings_editor,
        available_items: ItemSet::empty(),
        room_id: legacy.room_id,
        room_code: legacy.room_code,
        round_id: legacy.round_id,
        revision: legacy.revision,
        tick: legacy.tick,
        rules_version: legacy.rules_version,
        map_id: legacy.map_id,
        round_number: RoundCount::new(legacy.round_number.get())
            .map_err(|_| ProjectionError::InvalidClassicBinding)?,
        scores: Scores {
            one: Wins::new(legacy.scores.one.get())
                .map_err(|_| ProjectionError::InvalidClassicBinding)?,
            two: Wins::new(legacy.scores.two.get())
                .map_err(|_| ProjectionError::InvalidClassicBinding)?,
        },
        last_round: legacy.last_round.map(|round| CompletedRound {
            round_id: round.round_id,
            round_number: round.round_number,
            ended_at: round.ended_at,
            result: round.result,
        }),
        seats: legacy.seats.map(|seat| SeatStatus {
            seat: seat.seat,
            occupied: seat.occupied,
            ready: seat.ready,
            rematch_ready: seat.rematch_ready,
            connection: connection(seat.connection),
        }),
        phase: phase(legacy.phase),
    })
}

fn connection(value: crate::protocol::v6::ConnectionStatus) -> ConnectionStatus {
    match value {
        crate::protocol::v6::ConnectionStatus::Vacant => ConnectionStatus::Vacant,
        crate::protocol::v6::ConnectionStatus::Connected => ConnectionStatus::Connected,
        crate::protocol::v6::ConnectionStatus::Disconnected { expires_at } => {
            ConnectionStatus::Disconnected { expires_at }
        }
        crate::protocol::v6::ConnectionStatus::Expired => ConnectionStatus::Expired,
    }
}

fn phase(value: crate::protocol::v6::RoomPhase) -> RoomPhase {
    match value {
        crate::protocol::v6::RoomPhase::Waiting => RoomPhase::Waiting,
        crate::protocol::v6::RoomPhase::Countdown { kind, ends_at } => RoomPhase::Countdown {
            kind: match kind {
                crate::protocol::v6::CountdownKind::Initial => CountdownKind::Initial,
                crate::protocol::v6::CountdownKind::InterRound => CountdownKind::InterRound,
            },
            ends_at,
        },
        crate::protocol::v6::RoomPhase::Running => RoomPhase::Running,
        crate::protocol::v6::RoomPhase::Result { hold_until } => RoomPhase::Result { hold_until },
        crate::protocol::v6::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => RoomPhase::MatchResult {
            ended_at,
            result: match_result(result),
            rematch_at,
        },
        crate::protocol::v6::RoomPhase::Interrupted { ended_at, reason } => {
            RoomPhase::Interrupted {
                ended_at,
                reason: interrupt(reason),
            }
        }
    }
}

fn match_result(value: crate::protocol::v6::MatchResult) -> MatchResult {
    match value {
        crate::protocol::v6::MatchResult::Winner { seat, reason } => MatchResult::Winner {
            seat,
            reason: match reason {
                crate::protocol::v6::MatchWinnerReason::RoundWins => MatchWinnerReason::RoundWins,
                crate::protocol::v6::MatchWinnerReason::Forfeit => MatchWinnerReason::Forfeit,
            },
        },
        crate::protocol::v6::MatchResult::Abandoned => MatchResult::Abandoned,
        crate::protocol::v6::MatchResult::Interrupted { reason } => MatchResult::Interrupted {
            reason: interrupt(reason),
        },
    }
}

fn interrupt(value: crate::protocol::v6::MatchInterruptReason) -> MatchInterruptReason {
    match value {
        crate::protocol::v6::MatchInterruptReason::ConnectionLost => {
            MatchInterruptReason::ConnectionLost
        }
        crate::protocol::v6::MatchInterruptReason::ServerOverloaded => {
            MatchInterruptReason::ServerOverloaded
        }
        crate::protocol::v6::MatchInterruptReason::ServerStopped => {
            MatchInterruptReason::ServerStopped
        }
        crate::protocol::v6::MatchInterruptReason::InternalError => {
            MatchInterruptReason::InternalError
        }
    }
}
