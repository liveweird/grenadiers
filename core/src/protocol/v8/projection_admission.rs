use crate::protocol::{Counter, Seat};

use crate::game::GenerationRecord;

use super::projection_binding::{phase_allows_null_body, validate_expanded_room};
use super::projection_recipient::recipient_state;
use super::{
    ConnectionStatus, ExpandedCapture, GameServerMessage, ProjectionError, ResumeToken, RoomView,
    bind_expanded,
};

pub struct AdmissionProjection<'a> {
    pub room: &'a RoomView,
    pub state_epoch: Option<Counter>,
    pub capture: Option<&'a ExpandedCapture>,
    pub generation: Option<GenerationRecord>,
}

pub struct PersonalAdmission {
    pub recipient: Seat,
    pub last_action_id: Option<Counter>,
    pub sequence: Counter,
    pub resume_token: ResumeToken,
}

pub fn prepare_admission(
    projection: AdmissionProjection<'_>,
    personal: PersonalAdmission,
) -> Result<String, ProjectionError> {
    let AdmissionProjection {
        room,
        state_epoch,
        capture,
        generation,
    } = projection;
    let PersonalAdmission {
        recipient,
        last_action_id,
        sequence,
        resume_token,
    } = personal;
    validate_expanded_room(room)?;
    let seat = &room.seats[seat_index(recipient)];
    if seat.seat != recipient || !seat.occupied || seat.connection != ConnectionStatus::Connected {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let state = match (state_epoch, capture) {
        (Some(epoch), Some(capture)) => {
            let publication = bind_expanded(room, epoch, capture, generation)?;
            Some(recipient_state(&publication, recipient))
        }
        (None, None) if generation.is_none() && phase_allows_null_body(&room.phase) => None,
        _ => return Err(ProjectionError::InvalidExpandedBinding),
    };
    if last_action_id.is_some() && state_epoch != Some(room.round_id) {
        return Err(ProjectionError::InvalidExpandedBinding);
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

// Compatibility wrapper for callers that still use the original positional API.
#[allow(clippy::too_many_arguments)]
pub fn prepare_expanded_admission(
    room: &RoomView,
    state_epoch: Option<Counter>,
    capture: Option<&ExpandedCapture>,
    generation: Option<GenerationRecord>,
    recipient: Seat,
    last_action_id: Option<Counter>,
    sequence: Counter,
    resume_token: ResumeToken,
) -> Result<String, ProjectionError> {
    prepare_admission(
        AdmissionProjection {
            room,
            state_epoch,
            capture,
            generation,
        },
        PersonalAdmission {
            recipient,
            last_action_id,
            sequence,
            resume_token,
        },
    )
}

const fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::One => 0,
        Seat::Two => 1,
    }
}
