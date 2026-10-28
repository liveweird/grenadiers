use crate::game::GenerationRecord;
use crate::protocol::Counter;
use crate::room::SeatControl;

use super::{
    ConnectionStatus, ExpandedCapture, GameServerMessage, ProjectionError, ResumeToken, RoomView,
    Seat, bind_expanded, project_recipient,
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
    super::validate_room_view(projection.room)?;
    let index = personal.recipient.index();
    let Some(status) = projection.room.seats.get(index) else {
        return Err(ProjectionError::InvalidExpandedBinding);
    };
    if status.seat != personal.recipient
        || !status.occupied
        || status.connection != ConnectionStatus::Connected
        || projection.room.configuration.domain().controllers()[index] != SeatControl::Human
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    let state = match (projection.state_epoch, projection.capture) {
        (Some(epoch), Some(capture)) => {
            let publication =
                bind_expanded(projection.room, epoch, capture, projection.generation)?;
            Some(project_recipient(&publication, personal.recipient)?.state)
        }
        (None, None)
            if projection.generation.is_none()
                && super::projection_binding::phase_allows_null_body(&projection.room.phase) =>
        {
            None
        }
        _ => return Err(ProjectionError::InvalidExpandedBinding),
    };
    if personal.last_action_id.is_some() && projection.state_epoch != Some(projection.room.round_id)
    {
        return Err(ProjectionError::InvalidExpandedBinding);
    }
    super::encode_server_message(&GameServerMessage::Admitted {
        sequence: personal.sequence,
        room: projection.room.clone(),
        you_seat: personal.recipient,
        resume_token: personal.resume_token,
        last_action_id: personal.last_action_id,
        state,
    })
    .map_err(|_| ProjectionError::EncodingFailed)
}
