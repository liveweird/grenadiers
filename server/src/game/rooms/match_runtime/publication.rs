use grenadiers_core::{
    game::StepReport,
    protocol::v13::{
        RoomMetadata, RoomView, capture_expanded, prepare_snapshot_pair, project_expanded_room,
    },
    room::MatchLifecycle,
};

use super::super::types::RoomIdentity;
use super::error::MatchRuntimeError;
use super::{
    SeatFrames,
    runtime::{MatchNotice, SimRound},
};
use grenadiers_core::protocol::v13::RoomPhase;

pub(super) fn room_view(
    identity: &RoomIdentity,
    lifecycle: &MatchLifecycle,
    revision: u32,
    settings_revision: u32,
    settings_editor: Option<grenadiers_core::game::SeatId>,
) -> Result<RoomView, MatchRuntimeError> {
    let revision = grenadiers_core::protocol::Counter::new(revision)
        .map_err(|_| MatchRuntimeError::CapacityReached)?;
    project_expanded_room(
        RoomMetadata {
            settings: wire_settings(lifecycle.settings())?,
            settings_revision: grenadiers_core::protocol::Counter::new(settings_revision)
                .map_err(|_| MatchRuntimeError::CapacityReached)?,
            settings_editor: settings_editor.map(wire_seat),
            room_id: identity.id.clone(),
            room_code: identity.code.clone(),
            revision,
            rules_version: identity.profile.rules_version(),
            map_id: identity.profile.map_id(),
            mode: identity.mode,
        },
        lifecycle,
    )
    .map_err(|_| MatchRuntimeError::Publication)
}

pub(super) const fn wire_seat(
    seat: grenadiers_core::game::SeatId,
) -> grenadiers_core::protocol::Seat {
    match seat {
        grenadiers_core::game::SeatId::One => grenadiers_core::protocol::Seat::One,
        grenadiers_core::game::SeatId::Two => grenadiers_core::protocol::Seat::Two,
    }
}

pub(super) const fn phase_has_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Running | RoomPhase::Result { .. } | RoomPhase::MatchResult { .. }
    )
}

pub(super) fn snapshot(
    identity: &RoomIdentity,
    lifecycle: &MatchLifecycle,
    revision: u32,
    settings_revision: u32,
    settings_editor: Option<grenadiers_core::game::SeatId>,
    sim: &SimRound,
    reports: &[StepReport],
) -> Result<MatchNotice, MatchRuntimeError> {
    let generation_record_matches = match identity.profile {
        #[cfg(test)]
        super::super::profile::RoomProfile::FixedClassicV1 => sim.generation_record.is_none(),
        super::super::profile::RoomProfile::GeneratedClassicV2
        | super::super::profile::RoomProfile::GeneratedClassicV3
        | super::super::profile::RoomProfile::GeneratedConfigurableV4
        | super::super::profile::RoomProfile::ExpandedV5 => sim.generation_record.is_some(),
    };
    if !generation_record_matches {
        return Err(MatchRuntimeError::Publication);
    }
    let room = room_view(
        identity,
        lifecycle,
        revision,
        settings_revision,
        settings_editor,
    )?;
    if sim.state.rules() != &identity.profile.rules_config(lifecycle.settings()) {
        return Err(MatchRuntimeError::Publication);
    }
    let capture =
        capture_expanded(&sim.state, reports).map_err(|_| MatchRuntimeError::Publication)?;
    let epoch = grenadiers_core::protocol::Counter::new(
        u32::try_from(sim.epoch).map_err(|_| MatchRuntimeError::CapacityReached)?,
    )
    .map_err(|_| MatchRuntimeError::CapacityReached)?;
    let bound = grenadiers_core::protocol::v13::bind_expanded(
        &room,
        epoch,
        &capture,
        sim.generation_record,
    )
    .map_err(|_| MatchRuntimeError::Publication)?;
    let cpu_observation = if matches!(identity.mode, grenadiers_core::room::RoomMode::Solo(_)) {
        Some(
            grenadiers_core::protocol::v13::observe_cpu_recipient(
                &bound,
                grenadiers_core::protocol::Seat::Two,
                grenadiers_core::cpu::WorldId {
                    room_instance: identity.registry_serial,
                    physical_round: sim.epoch,
                },
            )
            .map_err(|_| MatchRuntimeError::Publication)?,
        )
    } else {
        None
    };
    let encoded = prepare_snapshot_pair(&bound).map_err(|_| MatchRuntimeError::Publication)?;
    Ok(MatchNotice {
        room,
        frames: Some(SeatFrames::new(encoded)),
        cpu_observation,
    })
}

fn wire_settings(
    settings: grenadiers_core::room::RoomSettings,
) -> Result<grenadiers_core::protocol::v13::GameSettings, MatchRuntimeError> {
    grenadiers_core::protocol::v13::GameSettings::new(
        speed_level(settings.starting_speed()),
        settings.turn_assist_units(),
        settings.wins_to_finish(),
        u16::try_from(settings.bomb_fuse_ticks()).map_err(|_| MatchRuntimeError::Publication)?,
        grenadiers_core::protocol::v13::ItemSet::from_mask(
            grenadiers_core::protocol::v13::ItemMask::new(settings.enabled_items().bits())
                .map_err(|_| MatchRuntimeError::Publication)?,
        ),
    )
    .map_err(|_| MatchRuntimeError::Publication)
}

const fn speed_level(speed: grenadiers_core::game::SpeedLevel) -> u8 {
    match speed {
        grenadiers_core::game::SpeedLevel::Base => 0,
        grenadiers_core::game::SpeedLevel::One => 1,
        grenadiers_core::game::SpeedLevel::Two => 2,
        grenadiers_core::game::SpeedLevel::Three => 3,
        grenadiers_core::game::SpeedLevel::Four => 4,
    }
}

#[cfg(test)]
pub(super) fn phase_has_frozen_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Result { .. } | RoomPhase::MatchResult { .. }
    )
}
