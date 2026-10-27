use crate::protocol::Seat;

use super::{BoundPublication, GameServerMessage, ProjectionError};

pub fn prepare_snapshot_pair(
    publication: &BoundPublication,
) -> Result<[String; 2], ProjectionError> {
    Ok([
        prepare(publication, Seat::One)?,
        prepare(publication, Seat::Two)?,
    ])
}

fn prepare(publication: &BoundPublication, recipient: Seat) -> Result<String, ProjectionError> {
    let state = crate::protocol::v11::projection_recipient::recipient_state(
        &publication.physics,
        recipient,
    );
    let events = crate::protocol::v11::projection_recipient::recipient_events(
        &publication.physics,
        recipient,
    )?;
    super::encode_server_message(&GameServerMessage::Snapshot {
        room: publication.room.clone(),
        state,
        events,
    })
    .map_err(|_| ProjectionError::EncodingFailed)
}
