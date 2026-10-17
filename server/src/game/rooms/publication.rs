use std::sync::Arc;

use grenadiers_core::{
    game::{State, StepReport},
    protocol::{
        Counter, GameServerMessage, InterruptReason as WireInterrupt, RoomPhase, RoomView, Seat,
        SeatStatus, Tick, WireRoundResult, encode_server_message, project_events, project_state,
    },
    room::{InterruptReason, Phase, RoomLifecycle},
};

use super::types::RoomIdentity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PublicationError;

pub(super) fn room_view(
    identity: &RoomIdentity,
    lifecycle: &RoomLifecycle,
    revision: u32,
) -> Result<RoomView, PublicationError> {
    let seats = grenadiers_core::game::SeatId::ALL.map(|seat| {
        let state = lifecycle.seats()[seat.index()];
        SeatStatus {
            seat: wire_seat(seat),
            occupied: state.occupied(),
            ready: state.ready(),
        }
    });
    let phase = match lifecycle.phase() {
        Phase::Waiting => RoomPhase::Waiting,
        Phase::Countdown { ends_at } => RoomPhase::Countdown {
            ends_at: wire_tick(ends_at)?,
        },
        Phase::Running => RoomPhase::Running,
        Phase::Result { ended_at, result } => RoomPhase::Result {
            ended_at: wire_tick(ended_at)?,
            result: project_result(result),
        },
        Phase::Interrupted { ended_at, reason } => RoomPhase::Interrupted {
            ended_at: wire_tick(ended_at)?,
            reason: match reason {
                InterruptReason::ParticipantLeft => WireInterrupt::ParticipantLeft,
                InterruptReason::ConnectionLost => WireInterrupt::ConnectionLost,
                InterruptReason::ServerOverloaded => WireInterrupt::ServerOverloaded,
                InterruptReason::ServerStopped => WireInterrupt::ServerStopped,
                InterruptReason::InternalError => WireInterrupt::InternalError,
            },
        },
    };
    Ok(RoomView {
        room_id: identity.id.clone(),
        room_code: identity.code.clone(),
        round_id: Counter::new(
            u32::try_from(lifecycle.round_attempt()).map_err(|_| PublicationError)?,
        )
        .map_err(|_| PublicationError)?,
        revision: Counter::new(revision).map_err(|_| PublicationError)?,
        tick: wire_tick(lifecycle.tick())?,
        rules_version: identity.profile.rules_version(),
        map_id: identity.profile.map_id(),
        seats,
        phase,
    })
}

pub(super) fn snapshot(
    identity: &RoomIdentity,
    lifecycle: &RoomLifecycle,
    revision: u32,
    state: &State,
    reports: &[StepReport],
) -> Result<(RoomView, Arc<str>), PublicationError> {
    let room = room_view(identity, lifecycle, revision)?;
    let projected = project_state(&state.public_view()).map_err(|_| PublicationError)?;
    if projected.rules_version != identity.profile.rules_version()
        || projected.tick.get() != lifecycle.tick().0
        || projected.result != lifecycle_result(lifecycle.phase())
    {
        return Err(PublicationError);
    }
    let events = project_events(reports).map_err(|_| PublicationError)?;
    let message = GameServerMessage::Snapshot {
        room: room.clone(),
        state: projected.state,
        events,
    };
    let encoded = encode_server_message(&message).map_err(|_| PublicationError)?;
    Ok((room, Arc::from(encoded)))
}

fn lifecycle_result(phase: Phase) -> Option<WireRoundResult> {
    match phase {
        Phase::Result { result, .. } => Some(project_result(result)),
        _ => None,
    }
}

fn project_result(result: grenadiers_core::game::RoundResult) -> WireRoundResult {
    use grenadiers_core::game::{DrawReason, RoundResult};
    match result {
        RoundResult::Winner(seat) => WireRoundResult::Winner {
            seat: wire_seat(seat),
        },
        RoundResult::Draw(DrawReason::MutualElimination) => WireRoundResult::Draw {
            reason: grenadiers_core::protocol::DrawReason::MutualElimination,
        },
        RoundResult::Draw(DrawReason::Timeout) => WireRoundResult::Draw {
            reason: grenadiers_core::protocol::DrawReason::Timeout,
        },
    }
}

fn wire_tick(tick: grenadiers_core::game::Tick) -> Result<Tick, PublicationError> {
    Tick::new(tick.0).map_err(|_| PublicationError)
}

pub(super) const fn wire_seat(seat: grenadiers_core::game::SeatId) -> Seat {
    match seat {
        grenadiers_core::game::SeatId::One => Seat::One,
        grenadiers_core::game::SeatId::Two => Seat::Two,
    }
}
