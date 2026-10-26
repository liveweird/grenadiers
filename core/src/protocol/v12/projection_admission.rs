use crate::game::GenerationRecord;
use crate::protocol::{Counter, Seat};

use super::projection::checked_common_room;
use super::{
    ConnectionStatus, ExpandedCapture, GameServerMessage, ProjectionError, ResumeToken, RoomMode,
    RoomView, bind_expanded,
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
    let common = checked_common_room(room)?;
    if matches!(room.mode, RoomMode::Solo { .. }) && recipient != Seat::One {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let seat = &room.seats[seat_index(recipient)];
    if seat.seat != recipient || !seat.occupied || seat.connection != ConnectionStatus::Connected {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let state = match (state_epoch, capture) {
        (Some(epoch), Some(capture)) => {
            let publication = bind_expanded(room, epoch, capture, generation)?;
            Some(crate::protocol::v11::projection_recipient::recipient_state(
                &publication.physics,
                recipient,
            ))
        }
        (None, None)
            if generation.is_none()
                && crate::protocol::v8::projection_binding::phase_allows_null_body(
                    &common.phase,
                ) =>
        {
            None
        }
        _ => return Err(ProjectionError::InvalidExpandedBinding),
    };
    if last_action_id.is_some() && state_epoch != Some(room.round_id) {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    super::encode_server_message(&GameServerMessage::Admitted {
        sequence,
        room: room.clone(),
        you_seat: recipient,
        resume_token,
        last_action_id,
        state,
    })
    .map_err(|_| ProjectionError::EncodingFailed)
}

const fn seat_index(seat: Seat) -> usize {
    match seat {
        Seat::One => 0,
        Seat::Two => 1,
    }
}
