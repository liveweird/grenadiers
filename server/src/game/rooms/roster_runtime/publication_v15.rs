use grenadiers_core::{
    game::{GenerationRecord, State, StepReport, StormCloudConfig},
    protocol::v15,
    room::{EnvironmentRoomConfiguration, MatchPhase, RosterMatchLifecycle},
};

use super::{
    error::RosterRuntimeError,
    publication,
    types::{RosterIdentity, RosterNotice, RosterPublication, RosterSeatFrames},
};

pub(super) fn room_view(
    identity: &RosterIdentity,
    lifecycle: &RosterMatchLifecycle,
    environment: StormCloudConfig,
    revision: u32,
    settings_revision: u32,
) -> Result<v15::RoomView, RosterRuntimeError> {
    let legacy = publication::room_view(identity, lifecycle, revision, settings_revision)?;
    let room = v15::RoomView {
        settings: v15::RoomSettings::new(
            legacy.settings,
            v15::Environment {
                storm_cloud: environment == StormCloudConfig::Single,
            },
        ),
        settings_revision: legacy.settings_revision,
        settings_editor: legacy.settings_editor,
        available_items: legacy.available_items,
        configuration: v15::RoomConfiguration::from_domain(EnvironmentRoomConfiguration::new(
            identity.configuration,
            environment,
        )),
        room_id: legacy.room_id,
        room_code: legacy.room_code,
        round_id: legacy.round_id,
        revision: legacy.revision,
        tick: legacy.tick,
        rules_version: v15::RulesVersion::ENVIRONMENT_V7,
        map_id: legacy.map_id,
        round_number: legacy.round_number,
        scores: legacy.scores,
        last_round: legacy.last_round,
        seats: legacy.seats,
        phase: phase(lifecycle.phase())?,
    };
    Ok(room)
}

pub(super) fn snapshot(
    room: v15::RoomView,
    epoch: u64,
    state: &State,
    reports: &[StepReport],
    generation: GenerationRecord,
) -> Result<RosterNotice, RosterRuntimeError> {
    let capture =
        v15::capture_expanded(state, reports).map_err(|_| RosterRuntimeError::Publication)?;
    let publication =
        v15::bind_expanded(&room, publication::counter64(epoch)?, &capture, generation)
            .map_err(|_| RosterRuntimeError::Publication)?;
    let encoded =
        v15::prepare_snapshots(&publication).map_err(|_| RosterRuntimeError::Publication)?;
    let frames = RosterSeatFrames::new_v15(encoded).ok_or(RosterRuntimeError::Publication)?;
    Ok(RosterNotice {
        publication: RosterPublication::Environment(room, Some(std::sync::Arc::new(publication))),
        frames: Some(frames),
    })
}

pub(super) fn view_notice(room: v15::RoomView) -> RosterNotice {
    RosterNotice {
        publication: RosterPublication::Environment(room, None),
        frames: None,
    }
}

fn phase(value: MatchPhase) -> Result<v15::RoomPhase, RosterRuntimeError> {
    use grenadiers_core::room::CountdownKind;
    Ok(match value {
        MatchPhase::Waiting => v15::RoomPhase::Waiting,
        MatchPhase::Countdown { kind, ends_at } => v15::RoomPhase::Countdown {
            kind: match kind {
                CountdownKind::Initial => v15::CountdownKind::Initial,
                CountdownKind::InterRound => v15::CountdownKind::InterRound,
            },
            ends_at: grenadiers_core::protocol::Tick::new(ends_at.0)
                .map_err(|_| RosterRuntimeError::CapacityReached)?,
        },
        MatchPhase::Running => v15::RoomPhase::Running,
        MatchPhase::Result { next_countdown_at } => v15::RoomPhase::Result {
            hold_until: grenadiers_core::protocol::Tick::new(next_countdown_at.0)
                .map_err(|_| RosterRuntimeError::CapacityReached)?,
        },
        MatchPhase::MatchResult { .. } | MatchPhase::Interrupted { .. } => {
            let legacy = publication::phase(value)?;
            match legacy {
                grenadiers_core::protocol::v14::RoomPhase::MatchResult {
                    ended_at,
                    result,
                    rematch_at,
                } => v15::RoomPhase::MatchResult {
                    ended_at,
                    result,
                    rematch_at,
                },
                grenadiers_core::protocol::v14::RoomPhase::Interrupted { ended_at, reason } => {
                    v15::RoomPhase::Interrupted { ended_at, reason }
                }
                _ => unreachable!(),
            }
        }
    })
}
