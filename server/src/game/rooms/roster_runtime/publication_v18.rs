use grenadiers_core::{
    game::{GenerationRecord, State, StepReport},
    protocol::v18,
    room::{MatchPhase, RosterMatchLifecycle, SpikeRoomConfiguration},
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
) -> Result<v18::RoomView, RosterRuntimeError> {
    let EnvironmentProfile::SpikeV10 {
        cloud,
        lairs,
        spike_pits,
    } = profile
    else {
        return Err(RosterRuntimeError::UnsupportedConfiguration);
    };
    let base = grenadiers_core::room::LairRoomConfiguration::new(
        identity.configuration,
        cloud,
        lairs.as_legacy(),
    );
    let configuration = SpikeRoomConfiguration::new(base, spike_pits);
    let room = v18::RoomView {
        settings: v18::RoomSettings::new(
            publication::settings(lifecycle.settings())?,
            environment(cloud, lairs, spike_pits),
        ),
        settings_revision: publication::counter(settings_revision)?,
        settings_editor: (!settings_locked)
            .then(|| lifecycle.settings_editor())
            .flatten()
            .map(super::types::wire_seat_v18),
        available_items: publication::available_items()?,
        configuration: v18::RoomConfiguration::from_domain(configuration)
            .map_err(|_| RosterRuntimeError::Publication)?,
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: publication::counter64(lifecycle.round_attempt())?,
        revision: publication::counter(revision)?,
        tick: publication::tick(lifecycle.tick())?,
        rules_version: v18::RulesVersion::ARENA_V10,
        map_id: v18::MapId::GeneratedV2,
        round_number: v18::RoundCount::new(lifecycle.actual_round_number())
            .map_err(|_| RosterRuntimeError::CapacityReached)?,
        scores: publication::scores(lifecycle)?,
        last_round: lifecycle
            .latest_completed()
            .map(publication::completed)
            .transpose()?,
        seats: publication::seats(lifecycle)?,
        phase: phase(lifecycle.phase())?,
    };
    v18::validate_room_view(&room).map_err(|_| RosterRuntimeError::Publication)?;
    Ok(room)
}

pub(super) fn snapshot(
    room: v18::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v18::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v18::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v18::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v18(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::Spike(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v18::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::Spike(room, None),
        frames: None,
    }
}

pub(super) const fn environment(
    cloud: grenadiers_core::game::StormCloudConfig,
    lairs: grenadiers_core::room::RespawnLairTuning,
    spike_pits: grenadiers_core::room::SpikePitTuning,
) -> v18::Environment {
    v18::Environment {
        storm_cloud: matches!(cloud, grenadiers_core::game::StormCloudConfig::Single),
        lairs: v18::Lairs {
            enabled: lairs.enabled(),
            spawn_seconds: lairs.settings().spawn_seconds(),
            max_living_mobs: 1,
        },
        spike_pits: v18::SpikePits {
            enabled: spike_pits.enabled(),
            warning_ticks: spike_pits.settings().warning_ticks() as u16,
            raised_ticks: spike_pits.settings().raised_ticks() as u16,
        },
    }
}

pub(super) fn phase(value: MatchPhase) -> Result<v18::RoomPhase, RosterRuntimeError> {
    let legacy = publication::phase(value)?;
    Ok(match legacy {
        grenadiers_core::protocol::v14::RoomPhase::Waiting => v18::RoomPhase::Waiting,
        grenadiers_core::protocol::v14::RoomPhase::Countdown { kind, ends_at } => {
            v18::RoomPhase::Countdown { kind, ends_at }
        }
        grenadiers_core::protocol::v14::RoomPhase::Running => v18::RoomPhase::Running,
        grenadiers_core::protocol::v14::RoomPhase::Result { hold_until } => {
            v18::RoomPhase::Result { hold_until }
        }
        grenadiers_core::protocol::v14::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        } => v18::RoomPhase::MatchResult {
            ended_at,
            result,
            rematch_at,
        },
        grenadiers_core::protocol::v14::RoomPhase::Interrupted { ended_at, reason } => {
            v18::RoomPhase::Interrupted { ended_at, reason }
        }
    })
}
