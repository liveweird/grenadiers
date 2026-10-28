use grenadiers_core::{
    game::{GenerationRecord, SeatId, State, StepReport},
    protocol::{Counter, v13, v14},
};

mod room;

pub(in crate::game::rooms) fn room_view(
    legacy: &v13::RoomView,
) -> Result<v14::RoomView, v14::ProjectionError> {
    room::project(legacy)
}

pub(in crate::game::rooms) fn snapshot(
    legacy: &v13::RoomView,
    epoch: Counter,
    state: &State,
    reports: &[StepReport],
    generation: Option<GenerationRecord>,
) -> Result<[String; 2], v14::ProjectionError> {
    let room = room_view(legacy)?;
    let capture = v14::capture_expanded(state, reports)?;
    let publication = v14::bind_expanded(&room, epoch, &capture, generation)?;
    let frames = v14::prepare_snapshots(&publication)?;
    match frames.as_slice() {
        [(v14::Seat::One, one), (v14::Seat::Two, two)] => Ok([one.clone(), two.clone()]),
        _ => Err(v14::ProjectionError::InvalidExpandedBinding),
    }
}

pub(in crate::game::rooms) fn room_state(
    legacy: &v13::RoomView,
) -> Result<String, v14::ProjectionError> {
    v14::encode_server_message(&v14::GameServerMessage::RoomState {
        room: room_view(legacy)?,
    })
    .map_err(|_| v14::ProjectionError::EncodingFailed)
}

#[allow(clippy::too_many_arguments)]
pub(in crate::game::rooms) fn admission(
    legacy: &v13::RoomView,
    epoch: Option<Counter>,
    capture: Option<&v14::ExpandedCapture>,
    generation: Option<GenerationRecord>,
    recipient: SeatId,
    last_action: Option<Counter>,
    sequence: Counter,
    token: v14::ResumeToken,
) -> Result<String, v14::ProjectionError> {
    let room = room_view(legacy)?;
    let recipient = room::seat_id(recipient)?;
    v14::prepare_admission(
        v14::AdmissionProjection {
            room: &room,
            state_epoch: epoch,
            capture,
            generation,
        },
        v14::PersonalAdmission {
            recipient,
            last_action_id: last_action,
            sequence,
            resume_token: token,
        },
    )
}

#[cfg(test)]
mod tests;
