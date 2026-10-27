use axum::extract::ws::WebSocket;
use grenadiers_core::protocol::{Counter, v13::V11ErrorCode as ErrorCode};

use super::{
    connection::ConnectionState,
    rooms::{Admission, RoomError},
    socket::send_error,
};
use crate::admission::{DenialOperation, DenialReason, GameplayOperation, RoomAdmissionPermit};

pub(super) async fn acquire(
    socket: &mut WebSocket,
    state: &ConnectionState,
    operation: GameplayOperation,
    sequence: Counter,
) -> Result<RoomAdmissionPermit, bool> {
    let now = std::time::Instant::now();
    let permit = state
        .peer_admission
        .check(state.peer_key, operation, now)
        .and_then(|()| {
            state
                .peer_admission
                .acquire_room_admission(state.peer_key, now)
        });
    match permit {
        Ok(permit) => Ok(permit),
        Err(denial) => {
            state
                .denial_telemetry
                .record(denial.operation, denial.reason, now);
            let code = if denial.reason == DenialReason::PeerRate {
                ErrorCode::RateLimited
            } else {
                ErrorCode::CapacityReached
            };
            Err(send_error(socket, code, Some(sequence)).await.is_ok())
        }
    }
}

pub(super) fn record_room_result(
    state: &ConnectionState,
    operation: DenialOperation,
    result: &Result<Admission, RoomError>,
) {
    let Err(error) = result else { return };
    let reason = match error {
        RoomError::RateLimited => Some(DenialReason::GlobalRate),
        RoomError::CapacityReached => Some(DenialReason::GlobalCapacity),
        RoomError::ResumeRejected => Some(DenialReason::ResumeRejected),
        _ => None,
    };
    if let Some(reason) = reason {
        state
            .denial_telemetry
            .record(operation, reason, std::time::Instant::now());
    }
}
