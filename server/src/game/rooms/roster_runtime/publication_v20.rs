use grenadiers_core::{
    game::{GenerationRecord, State, StepReport},
    protocol::v20,
    room::RosterMatchLifecycle,
};

use super::{
    error::RosterRuntimeError,
    publication,
    runtime::EnvironmentProfile,
    types::{RosterIdentity, RosterNotice, RosterPublication, RosterSeatFrames},
};

pub(super) fn room_view(
    identity: &RosterIdentity,
    lifecycle: &RosterMatchLifecycle,
    profile: EnvironmentProfile,
    revision: u32,
    settings_revision: u32,
    settings_locked: bool,
) -> Result<v20::RoomView, RosterRuntimeError> {
    super::publication_v19::room_view(
        identity,
        lifecycle,
        profile,
        revision,
        settings_revision,
        settings_locked,
    )
}

pub(super) fn snapshot(
    room: v20::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v20::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v20::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v20::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v20(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::SpikeV20(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v20::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::SpikeV20(room, None),
        frames: None,
    }
}
