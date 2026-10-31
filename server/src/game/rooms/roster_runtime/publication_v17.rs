use grenadiers_core::{
    game::{GenerationRecord, State, StepReport, StormCloudConfig},
    protocol::v17,
    room::{LairRoomConfiguration, MatchPhase, RespawnLairTuning, RosterMatchLifecycle},
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
) -> Result<v17::RoomView, RosterRuntimeError> {
    let EnvironmentProfile::RespawnV9 { cloud, tuning } = profile else {
        return Err(RosterRuntimeError::UnsupportedConfiguration);
    };
    let configuration =
        LairRoomConfiguration::new(identity.configuration, cloud, tuning.as_legacy());
    let room = v17::RoomView {
        settings: v17::RoomSettings::new(
            publication::settings(lifecycle.settings())?,
            environment(cloud, tuning),
        ),
        settings_revision: publication::counter(settings_revision)?,
        settings_editor: (!settings_locked)
            .then(|| lifecycle.settings_editor())
            .flatten()
            .map(super::types::wire_seat_v17),
        available_items: publication::available_items()?,
        configuration: v17::RoomConfiguration::from_domain(configuration)
            .map_err(|_| RosterRuntimeError::Publication)?,
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: publication::counter64(lifecycle.round_attempt())?,
        revision: publication::counter(revision)?,
        tick: publication::tick(lifecycle.tick())?,
        rules_version: v17::RulesVersion::ARENA_V9,
        map_id: v17::MapId::GeneratedV2,
        round_number: v17::RoundCount::new(lifecycle.actual_round_number())
            .map_err(|_| RosterRuntimeError::CapacityReached)?,
        scores: publication::scores(lifecycle)?,
        last_round: lifecycle
            .latest_completed()
            .map(publication::completed)
            .transpose()?,
        seats: publication::seats(lifecycle)?,
        phase: phase(lifecycle.phase())?,
    };
    v17::validate_room_view(&room).map_err(|_| RosterRuntimeError::Publication)?;
    Ok(room)
}

pub(super) fn snapshot(
    room: v17::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v17::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v17::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v17::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v17(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::Respawn(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v17::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::Respawn(room, None),
        frames: None,
    }
}

pub(super) const fn environment(
    cloud: StormCloudConfig,
    tuning: RespawnLairTuning,
) -> v17::Environment {
    v17::Environment {
        storm_cloud: matches!(cloud, StormCloudConfig::Single),
        lairs: v17::Lairs {
            enabled: tuning.enabled(),
            spawn_seconds: tuning.settings().spawn_seconds(),
            max_living_mobs: 1,
        },
    }
}

fn phase(value: MatchPhase) -> Result<v17::RoomPhase, RosterRuntimeError> {
    let legacy = publication::phase(value)?;
    Ok(match legacy {
        grenadiers_core::protocol::v14::RoomPhase::Waiting => v17::RoomPhase::Waiting,
        grenadiers_core::protocol::v14::RoomPhase::Countdown { kind, ends_at } => {
            v17::RoomPhase::Countdown { kind, ends_at }
        }
        grenadiers_core::protocol::v14::RoomPhase::Running => v17::RoomPhase::Running,
        grenadiers_core::protocol::v14::RoomPhase::Result { hold_until } => {
            v17::RoomPhase::Result { hold_until }
        }
        grenadiers_core::protocol::v14::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => v17::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        },
        grenadiers_core::protocol::v14::RoomPhase::Interrupted { ended_at, reason } => {
            v17::RoomPhase::Interrupted { ended_at, reason }
        }
    })
}
