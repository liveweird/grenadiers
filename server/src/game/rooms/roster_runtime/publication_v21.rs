use grenadiers_core::{
    game::{GenerationRecord, State, StepReport},
    protocol::v21,
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
) -> Result<v21::RoomView, RosterRuntimeError> {
    let legacy = super::publication_v20::room_view(
        identity,
        lifecycle,
        profile,
        revision,
        settings_revision,
        settings_locked,
    )?;
    let EnvironmentProfile::SpikeV10 {
        cloud,
        lairs,
        spike_pits,
    } = profile
    else {
        return Err(RosterRuntimeError::UnsupportedConfiguration);
    };
    let settings = v21::RoomSettings::new(
        publication::settings(lifecycle.settings())?,
        super::publication_v19::environment(cloud, lairs, spike_pits),
        lifecycle.settings().bomb_capacity_drop_multiplier(),
    );
    let room = v21::RoomView::from_v20(legacy, settings);
    v21::validate_room_view(&room).map_err(|_| RosterRuntimeError::Publication)?;
    Ok(room)
}

pub(super) fn snapshot(
    room: v21::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v21::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v21::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v21::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v20(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::SpikeV21(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v21::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::SpikeV21(room, None),
        frames: None,
    }
}
