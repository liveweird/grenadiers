use crate::game::GenerationRecord;
use crate::protocol::Counter;

use super::{GameServerMessage, ProjectionError, RoomView, Seat};

pub use crate::protocol::v19::{BoundPublication, ExpandedCapture, RecipientProjection};

pub fn capture_expanded(
    state: &crate::game::State,
    reports: &[crate::game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    crate::protocol::v19::capture_expanded(state, reports)
}

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: GenerationRecord,
) -> Result<BoundPublication, ProjectionError> {
    crate::protocol::v19::bind_expanded_with_punch_activation(
        room,
        state_epoch,
        capture,
        generation,
        crate::game::PunchActivationPolicy::ContactV2,
    )
}

pub fn project_recipient(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<RecipientProjection, ProjectionError> {
    crate::protocol::v19::project_recipient(publication, recipient)
}

pub fn prepare_snapshots(
    publication: &BoundPublication,
) -> Result<Vec<(Seat, String)>, ProjectionError> {
    let count = publication
        .room()
        .configuration
        .domain()
        .base()
        .base()
        .controllers()
        .len();
    [Seat::One, Seat::Two, Seat::Three, Seat::Four]
        .into_iter()
        .take(count)
        .map(|recipient| {
            let projected = project_recipient(publication, recipient)?;
            let encoded = super::encode_server_message(&GameServerMessage::Snapshot {
                room: publication.room().clone(),
                state: projected.state,
                events: projected.events,
            })
            .map_err(|_| ProjectionError::EncodingFailed)?;
            Ok((recipient, encoded))
        })
        .collect()
}
