use std::sync::Arc;

use grenadiers_core::{
    game::StepReport,
    protocol::v3::{
        GameServerMessage, RoomMetadata, RoomPhase, RoomView, encode_server_message,
        project_room_view,
    },
    protocol::{project_events, project_state},
    room::{MatchLifecycle, MatchPhase},
};

use super::super::types::RoomIdentity;
use super::error::MatchRuntimeError;
use super::runtime::{MatchNotice, SimRound};

pub(super) fn room_view(
    identity: &RoomIdentity,
    lifecycle: &MatchLifecycle,
    revision: u32,
) -> Result<RoomView, MatchRuntimeError> {
    let revision = grenadiers_core::protocol::Counter::new(revision)
        .map_err(|_| MatchRuntimeError::CapacityReached)?;
    project_room_view(
        RoomMetadata {
            room_id: identity.id.clone(),
            room_code: identity.code.clone(),
            revision,
            rules_version: identity.profile.rules_version(),
            map_id: identity.profile.map_id(),
        },
        lifecycle,
    )
    .map_err(|_| MatchRuntimeError::Publication)
}

pub(super) fn snapshot(
    identity: &RoomIdentity,
    lifecycle: &MatchLifecycle,
    revision: u32,
    sim: &SimRound,
    reports: &[StepReport],
) -> Result<MatchNotice, MatchRuntimeError> {
    let room = room_view(identity, lifecycle, revision)?;
    let projected =
        project_state(&sim.state.public_view()).map_err(|_| MatchRuntimeError::Publication)?;
    if projected.rules_version != identity.profile.rules_version() {
        return Err(MatchRuntimeError::Publication);
    }
    validate_state(lifecycle, &room, sim, projected.tick, projected.result)?;
    let events = project_events(reports).map_err(|_| MatchRuntimeError::Publication)?;
    let message = GameServerMessage::Snapshot {
        room: room.clone(),
        state: projected.state,
        events,
    };
    let encoded = encode_server_message(&message).map_err(|_| MatchRuntimeError::Publication)?;
    Ok(MatchNotice {
        room,
        snapshot: Some(Arc::from(encoded)),
    })
}

fn validate_state(
    lifecycle: &MatchLifecycle,
    room: &RoomView,
    sim: &SimRound,
    state_tick: grenadiers_core::protocol::Tick,
    state_result: Option<grenadiers_core::protocol::WireRoundResult>,
) -> Result<(), MatchRuntimeError> {
    if state_tick.get() > room.tick.get() {
        return Err(MatchRuntimeError::Publication);
    }
    match lifecycle.phase() {
        MatchPhase::Running => valid(
            sim.epoch == lifecycle.round_attempt()
                && state_tick.get() == lifecycle.tick().0
                && state_result.is_none(),
        ),
        MatchPhase::Result { .. } => validate_completed(room, sim, state_tick, state_result),
        MatchPhase::MatchResult { outcome, .. } if state_result.is_some() => {
            validate_completed(room, sim, state_tick, state_result)?;
            valid(state_tick.get() <= outcome.ended_at().0)
        }
        MatchPhase::MatchResult { outcome, .. } => valid(
            sim.epoch == lifecycle.round_attempt()
                && state_tick.get() == outcome.ended_at().0
                && state_result.is_none(),
        ),
        _ => Err(MatchRuntimeError::Publication),
    }
}

fn validate_completed(
    room: &RoomView,
    sim: &SimRound,
    state_tick: grenadiers_core::protocol::Tick,
    state_result: Option<grenadiers_core::protocol::WireRoundResult>,
) -> Result<(), MatchRuntimeError> {
    let Some(last) = room.last_round else {
        return Err(MatchRuntimeError::Publication);
    };
    valid(
        sim.epoch == u64::from(last.round_id.get())
            && state_tick == last.ended_at
            && state_result == Some(last.result),
    )
}

fn valid(condition: bool) -> Result<(), MatchRuntimeError> {
    condition
        .then_some(())
        .ok_or(MatchRuntimeError::Publication)
}

#[cfg(test)]
pub(super) fn phase_has_frozen_body(phase: &RoomPhase) -> bool {
    matches!(
        phase,
        RoomPhase::Result { .. } | RoomPhase::MatchResult { .. }
    )
}
