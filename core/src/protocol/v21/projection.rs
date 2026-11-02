use crate::game::GenerationRecord;
use crate::protocol::Counter;

use super::{GameServerMessage, ProjectionError, RoomView, Seat};

pub use crate::protocol::v20::{ExpandedCapture, RecipientProjection};

pub struct BoundPublication {
    room: RoomView,
    base: crate::protocol::v20::BoundPublication,
}

impl BoundPublication {
    pub(crate) const fn base(&self) -> &crate::protocol::v20::BoundPublication {
        &self.base
    }

    pub(crate) const fn room(&self) -> &RoomView {
        &self.room
    }
}

pub fn capture_expanded(
    state: &crate::game::State,
    reports: &[crate::game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    crate::protocol::v20::capture_expanded(state, reports)
}

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: Counter,
    capture: &ExpandedCapture,
    generation: GenerationRecord,
) -> Result<BoundPublication, ProjectionError> {
    super::validate_room_view(room)?;
    let legacy = room.legacy_view();
    let base = crate::protocol::v19::bind_expanded_with_expected_settings(
        &legacy,
        state_epoch,
        capture,
        generation,
        crate::game::WEIGHTED_GENERATOR_VERSION,
        room.settings
            .domain()
            .map_err(|()| ProjectionError::InvalidExpandedBinding)?,
        crate::game::PunchActivationPolicy::ContactV2,
    )?;
    Ok(BoundPublication {
        room: room.clone(),
        base,
    })
}

pub fn project_recipient(
    publication: &BoundPublication,
    recipient: Seat,
) -> Result<RecipientProjection, ProjectionError> {
    crate::protocol::v20::project_recipient(&publication.base, recipient)
}

pub fn prepare_snapshots(
    publication: &BoundPublication,
) -> Result<Vec<(Seat, String)>, ProjectionError> {
    let count = publication
        .room
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
                room: publication.room.clone(),
                state: projected.state,
                events: projected.events,
            })
            .map_err(|_| ProjectionError::EncodingFailed)?;
            Ok((recipient, encoded))
        })
        .collect()
}
