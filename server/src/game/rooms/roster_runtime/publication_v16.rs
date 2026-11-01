use grenadiers_core::{
    game::{GenerationRecord, State, StepReport, StormCloudConfig},
    protocol::v16,
    room::{LairRoomConfiguration, LairTuning, MatchPhase, RosterMatchLifecycle},
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
) -> Result<v16::RoomView, RosterRuntimeError> {
    let EnvironmentProfile::LairsV8 { cloud, tuning } = profile else {
        return Err(RosterRuntimeError::UnsupportedConfiguration);
    };
    let room = v16::RoomView {
        settings: v16::RoomSettings::new(
            publication::settings(lifecycle.settings())?,
            environment(cloud, tuning),
        ),
        settings_revision: publication::counter(settings_revision)?,
        settings_editor: (!settings_locked)
            .then(|| lifecycle.settings_editor())
            .flatten()
            .map(super::types::wire_seat_v16),
        available_items: publication::available_items()?,
        configuration: v16::RoomConfiguration::from_domain(LairRoomConfiguration::new(
            identity.configuration,
            cloud,
            tuning,
        ))
        .map_err(|_| RosterRuntimeError::UnsupportedConfiguration)?,
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: publication::counter64(lifecycle.round_attempt())?,
        revision: publication::counter(revision)?,
        tick: publication::tick(lifecycle.tick())?,
        rules_version: v16::RulesVersion::ENVIRONMENT_V8,
        map_id: v16::MapId::GeneratedV2,
        round_number: v16::RoundCount::new(lifecycle.actual_round_number())
            .map_err(|_| RosterRuntimeError::CapacityReached)?,
        scores: publication::scores(lifecycle)?,
        last_round: lifecycle
            .latest_completed()
            .map(publication::completed)
            .transpose()?,
        seats: publication::seats(lifecycle)?,
        phase: phase(lifecycle.phase())?,
    };
    v16::validate_room_view(&room).map_err(|_| RosterRuntimeError::Publication)?;
    Ok(room)
}

pub(super) fn snapshot(
    room: v16::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v16::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v16::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v16::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v16(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::Lairs(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v16::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::Lairs(room, None),
        frames: None,
    }
}

pub(super) const fn environment(cloud: StormCloudConfig, tuning: LairTuning) -> v16::Environment {
    v16::Environment {
        storm_cloud: matches!(cloud, StormCloudConfig::Single),
        lairs: v16::Lairs {
            enabled: tuning.enabled(),
            spawn_seconds: tuning.settings().spawn_seconds(),
            max_living_mobs: tuning.settings().mobs_per_lair(),
        },
    }
}

fn phase(value: MatchPhase) -> Result<v16::RoomPhase, RosterRuntimeError> {
    let legacy = publication::phase(value)?;
    Ok(match legacy {
        grenadiers_core::protocol::v14::RoomPhase::Waiting => v16::RoomPhase::Waiting,
        grenadiers_core::protocol::v14::RoomPhase::Countdown { kind, ends_at } => {
            v16::RoomPhase::Countdown { kind, ends_at }
        }
        grenadiers_core::protocol::v14::RoomPhase::Running => v16::RoomPhase::Running,
        grenadiers_core::protocol::v14::RoomPhase::Result { hold_until } => {
            v16::RoomPhase::Result { hold_until }
        }
        grenadiers_core::protocol::v14::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => v16::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        },
        grenadiers_core::protocol::v14::RoomPhase::Interrupted { ended_at, reason } => {
            v16::RoomPhase::Interrupted { ended_at, reason }
        }
    })
}
