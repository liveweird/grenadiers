use crate::protocol::{Counter, Seat};

use super::projection_binding::validate_classic_room;
use super::projection_recipient::recipient_state;
use super::{
    ClassicCapture, ConnectionStatus, GameServerMessage, ProjectionError, ResumeToken, RoomPhase,
    RoomView, bind_classic,
};

#[allow(clippy::too_many_arguments)]
pub fn prepare_classic_admission(
    room: &RoomView,
    state_epoch: Option<Counter>,
    capture: Option<&ClassicCapture>,
    recipient: Seat,
    last_action_id: Option<Counter>,
    sequence: Counter,
    resume_token: ResumeToken,
) -> Result<String, ProjectionError> {
    validate_classic_room(room)?;
    let seat = &room.seats[seat_index(recipient)];
    if seat.seat != recipient || seat.connection != ConnectionStatus::Connected {
        return Err(ProjectionError::InvalidClassicBinding);
    }
    let state = match (state_epoch, capture) {
        (Some(epoch), Some(capture)) => {
            let publication = bind_classic(room, epoch, capture)?;
            Some(recipient_state(&publication, recipient))
        }
        (None, None) if phase_allows_null_body(&room.phase) => None,
        _ => return Err(ProjectionError::InvalidClassicBinding),
    };
    if last_action_id.is_some() && state_epoch != Some(room.round_id) {
        return Err(ProjectionError::InvalidClassicBinding);
    }
    let message = GameServerMessage::Admitted {
        sequence,
        room: room.clone(),
        you_seat: recipient,
        resume_token,
        last_action_id,
        state,
    };
    super::encode_server_message(&message).map_err(|_| ProjectionError::EncodingFailed)
}

const fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::One => 0,
        Seat::Two => 1,
    }
}

fn phase_allows_null_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Waiting | RoomPhase::Countdown { .. } | RoomPhase::Interrupted { .. }
    ) || matches!(
        phase,
        RoomPhase::MatchResult {
            result: super::MatchResult::Interrupted {
                reason: super::MatchInterruptReason::InternalError
            },
            ..
        }
    )
}
