use grenadiers_core::{
    game::SeatId,
    protocol::{DrawReason, Seat as LegacySeat, WireRoundResult, v13, v14},
    room::{CpuDifficulty, RoomConfiguration},
};

pub(super) fn project(legacy: &v13::RoomView) -> Result<v14::RoomView, v14::ProjectionError> {
    let legacy_valid = v13::validate_room_view(legacy).is_ok();
    let legacy = normalized(legacy);
    if !legacy_valid {
        v13::validate_room_view(&legacy)
            .map_err(|_| v14::ProjectionError::InvalidExpandedBinding)?;
    }
    let room = v14::RoomView {
        settings: legacy.settings,
        settings_revision: legacy.settings_revision,
        settings_editor: legacy.settings_editor.map(seat),
        available_items: legacy.available_items,
        configuration: configuration(legacy.mode),
        room_id: legacy.room_id.clone(),
        room_code: legacy.room_code.clone(),
        round_id: legacy.round_id,
        revision: legacy.revision,
        tick: legacy.tick,
        rules_version: v14::RulesVersion::new(
            u8::try_from(legacy.rules_version.get())
                .map_err(|_| v14::ProjectionError::InvalidRules)?,
        )
        .map_err(|_| v14::ProjectionError::InvalidRules)?,
        map_id: map(legacy.map_id),
        round_number: legacy.round_number,
        scores: vec![
            v14::Score {
                seat: v14::Seat::One,
                wins: legacy.scores.one,
            },
            v14::Score {
                seat: v14::Seat::Two,
                wins: legacy.scores.two,
            },
        ],
        last_round: legacy.last_round.map(completed),
        seats: legacy.seats.into_iter().map(status).collect(),
        phase: phase(&legacy.phase),
    };
    v14::validate_room_view(&room)?;
    Ok(room)
}

fn normalized(legacy: &v13::RoomView) -> v13::RoomView {
    let mut room = legacy.clone();
    if room.settings_editor.is_some_and(|editor| {
        let connection = room.seats[match editor {
            LegacySeat::One => 0,
            LegacySeat::Two => 1,
        }]
        .connection;
        matches!(
            connection,
            v13::ConnectionStatus::Vacant | v13::ConnectionStatus::Expired
        )
    }) {
        room.settings_editor = None;
    }
    for status in &mut room.seats {
        if matches!(status.controller, v13::Controller::Cpu { .. }) {
            status.ready = matches!(
                room.phase,
                v13::RoomPhase::Waiting | v13::RoomPhase::Countdown { .. }
            );
            status.rematch_ready = matches!(
                room.phase,
                v13::RoomPhase::MatchResult { rematch_at, .. } if room.tick >= rematch_at
            );
        }
    }
    room
}

fn configuration(mode: v13::RoomMode) -> v14::RoomConfiguration {
    let domain = match mode {
        v13::RoomMode::HumanDuel => RoomConfiguration::human_duel(),
        v13::RoomMode::Solo {
            difficulty: v13::Difficulty::Easy,
        } => RoomConfiguration::solo(CpuDifficulty::Easy),
        v13::RoomMode::Solo {
            difficulty: v13::Difficulty::Standard,
        } => RoomConfiguration::solo(CpuDifficulty::Standard),
    };
    v14::RoomConfiguration::from_domain(domain)
}

const fn map(value: v13::MapId) -> v14::MapId {
    match value {
        v13::MapId::FixedV1 => v14::MapId::FixedV1,
        v13::MapId::GeneratedV1 => v14::MapId::GeneratedV1,
        v13::MapId::GeneratedV2 => v14::MapId::GeneratedV2,
    }
}

pub(super) const fn seat_id(value: SeatId) -> Result<v14::Seat, v14::ProjectionError> {
    match value {
        SeatId::One => Ok(v14::Seat::One),
        SeatId::Two => Ok(v14::Seat::Two),
        SeatId::Three | SeatId::Four => Err(v14::ProjectionError::InvalidExpandedBinding),
    }
}

const fn seat(value: LegacySeat) -> v14::Seat {
    match value {
        LegacySeat::One => v14::Seat::One,
        LegacySeat::Two => v14::Seat::Two,
    }
}

fn status(value: v13::SeatStatus) -> v14::SeatStatus {
    v14::SeatStatus {
        seat: seat(value.seat),
        occupied: value.occupied,
        ready: value.ready,
        rematch_ready: value.rematch_ready,
        controller: match value.controller {
            v13::Controller::Human => v14::Controller::Human,
            v13::Controller::Cpu {
                difficulty: v13::Difficulty::Easy,
            } => v14::Controller::Cpu {
                difficulty: v14::Difficulty::Easy,
            },
            v13::Controller::Cpu {
                difficulty: v13::Difficulty::Standard,
            } => v14::Controller::Cpu {
                difficulty: v14::Difficulty::Standard,
            },
        },
        connection: connection(value.connection),
    }
}

const fn connection(value: v13::ConnectionStatus) -> v14::ConnectionStatus {
    match value {
        v13::ConnectionStatus::Vacant => v14::ConnectionStatus::Vacant,
        v13::ConnectionStatus::Connected => v14::ConnectionStatus::Connected,
        v13::ConnectionStatus::Disconnected { expires_at } => {
            v14::ConnectionStatus::Disconnected { expires_at }
        }
        v13::ConnectionStatus::Expired => v14::ConnectionStatus::Expired,
        v13::ConnectionStatus::ServerControlled => v14::ConnectionStatus::ServerControlled,
    }
}

fn completed(value: v13::CompletedRound) -> v14::CompletedRound {
    v14::CompletedRound {
        round_id: value.round_id,
        round_number: value.round_number,
        ended_at: value.ended_at,
        result: match value.result {
            WireRoundResult::Winner { seat: winner } => {
                v14::RoundResult::Winner { seat: seat(winner) }
            }
            WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            } => v14::RoundResult::Draw {
                reason: v14::DrawReason::MutualElimination,
            },
            WireRoundResult::Draw {
                reason: DrawReason::Timeout,
            } => v14::RoundResult::Draw {
                reason: v14::DrawReason::Timeout,
            },
        },
    }
}

fn phase(value: &v13::RoomPhase) -> v14::RoomPhase {
    match *value {
        v13::RoomPhase::Waiting => v14::RoomPhase::Waiting,
        v13::RoomPhase::Countdown { kind, ends_at } => v14::RoomPhase::Countdown {
            kind: match kind {
                v13::CountdownKind::Initial => v14::CountdownKind::Initial,
                v13::CountdownKind::InterRound => v14::CountdownKind::InterRound,
            },
            ends_at,
        },
        v13::RoomPhase::Running => v14::RoomPhase::Running,
        v13::RoomPhase::Result { hold_until } => v14::RoomPhase::Result { hold_until },
        v13::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => v14::RoomPhase::MatchResult {
            ended_at,
            result: match_result(result),
            rematch_at,
        },
        v13::RoomPhase::Interrupted { ended_at, reason } => v14::RoomPhase::Interrupted {
            ended_at,
            reason: interrupt(reason),
        },
    }
}

const fn match_result(value: v13::MatchResult) -> v14::MatchResult {
    match value {
        v13::MatchResult::Winner {
            seat: winner,
            reason,
        } => v14::MatchResult::Winner {
            seat: seat(winner),
            reason: match reason {
                v13::MatchWinnerReason::RoundWins => v14::MatchWinnerReason::RoundWins,
                v13::MatchWinnerReason::Forfeit => v14::MatchWinnerReason::Forfeit,
            },
        },
        v13::MatchResult::Abandoned => v14::MatchResult::Abandoned,
        v13::MatchResult::Interrupted { reason } => v14::MatchResult::Interrupted {
            reason: interrupt(reason),
        },
    }
}

const fn interrupt(value: v13::MatchInterruptReason) -> v14::MatchInterruptReason {
    match value {
        v13::MatchInterruptReason::ConnectionLost => v14::MatchInterruptReason::ConnectionLost,
        v13::MatchInterruptReason::ServerOverloaded => v14::MatchInterruptReason::ServerOverloaded,
        v13::MatchInterruptReason::ServerStopped => v14::MatchInterruptReason::ServerStopped,
        v13::MatchInterruptReason::InternalError => v14::MatchInterruptReason::InternalError,
    }
}
