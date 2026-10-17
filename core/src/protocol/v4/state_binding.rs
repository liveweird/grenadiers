use super::super::{Counter, ProjectedRound, Seat};
use super::{
    ConnectionStatus, MatchInterruptReason, MatchResult, RoomPhase, RoomProjectionError, RoomView,
};

pub fn validate_state_binding(
    room: &RoomView,
    state_epoch: Counter,
    projected: &ProjectedRound,
) -> Result<(), RoomProjectionError> {
    super::projection_validation::validate_room_view(room)?;
    if projected.rules_version != room.rules_version {
        return Err(RoomProjectionError::InvalidStateBinding);
    }
    let valid = match room.phase {
        RoomPhase::Running => {
            state_epoch == room.round_id
                && projected.tick == room.tick
                && projected.result.is_none()
        }
        RoomPhase::Result { .. } => completed_binding(room, state_epoch, projected, None),
        RoomPhase::MatchResult { ended_at, .. } if projected.result.is_some() => {
            completed_binding(room, state_epoch, projected, Some(ended_at))
        }
        RoomPhase::MatchResult { ended_at, .. } => {
            state_epoch == room.round_id && projected.tick == ended_at && projected.result.is_none()
        }
        RoomPhase::Waiting | RoomPhase::Countdown { .. } | RoomPhase::Interrupted { .. } => false,
    };
    valid
        .then_some(())
        .ok_or(RoomProjectionError::InvalidStateBinding)
}

pub fn validate_admission(
    room: &RoomView,
    you_seat: Seat,
    state_epoch: Option<Counter>,
    projected: Option<&ProjectedRound>,
    last_bomb_action_id: Option<Counter>,
) -> Result<(), RoomProjectionError> {
    super::projection_validation::validate_room_view(room)?;
    let seat = &room.seats[match you_seat {
        Seat::One => 0,
        Seat::Two => 1,
    }];
    if seat.seat != you_seat || seat.connection != ConnectionStatus::Connected {
        return Err(RoomProjectionError::InvalidAdmission);
    }
    match (state_epoch, projected) {
        (Some(epoch), Some(projected)) => validate_state_binding(room, epoch, projected)?,
        (None, None) if phase_allows_null_body(&room.phase) => {}
        _ => return Err(RoomProjectionError::InvalidAdmission),
    }
    if last_bomb_action_id.is_some()
        && !matches!(state_epoch, Some(epoch) if epoch == room.round_id)
    {
        return Err(RoomProjectionError::InvalidAdmission);
    }
    Ok(())
}

fn completed_binding(
    room: &RoomView,
    state_epoch: Counter,
    projected: &ProjectedRound,
    outcome_ended_at: Option<super::super::Tick>,
) -> bool {
    room.last_round.is_some_and(|last| {
        state_epoch == last.round_id
            && projected.tick == last.ended_at
            && projected.result == Some(last.result)
            && outcome_ended_at.is_none_or(|ended_at| projected.tick <= ended_at)
    })
}

fn phase_allows_null_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Waiting | RoomPhase::Countdown { .. } | RoomPhase::Interrupted { .. }
    ) || matches!(
        phase,
        RoomPhase::MatchResult {
            result: MatchResult::Interrupted {
                reason: MatchInterruptReason::InternalError
            },
            ..
        }
    )
}
