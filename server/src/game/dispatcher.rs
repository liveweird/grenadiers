use std::sync::Arc;

use axum::extract::ws::WebSocket;
use grenadiers_core::protocol::v5::{
    GameClientMessage, GameServerMessage, V5ErrorCode as ErrorCode,
};
use grenadiers_core::protocol::{Counter, Direction as WireDirection, RoomId};
use tokio::time::Instant;

use super::{
    connection::{ConnectionState, Membership},
    rooms::{Admission, BindingKey, GameRooms, RoomAction, RoomError},
    socket::{fail_and_close, send_error, send_game},
};

pub(super) async fn dispatch(
    socket: &mut WebSocket,
    message: GameClientMessage,
    sequence: Counter,
    rooms: &Arc<GameRooms>,
    state: &mut ConnectionState,
) -> bool {
    match message {
        GameClientMessage::CreateRoom { .. } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let result = rooms
                .create_sequenced(binding, sequence, std::time::Instant::now())
                .await;
            admit(socket, result, binding, state, sequence).await
        }
        GameClientMessage::JoinRoom { room_code, .. } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let result = rooms
                .join_sequenced(&room_code, binding, sequence, std::time::Instant::now())
                .await;
            admit(socket, result, binding, state, sequence).await
        }
        GameClientMessage::ResumeRoom {
            room_id,
            resume_token,
            ..
        } => {
            if state.membership.is_some() {
                return reply_error(socket, ErrorCode::AlreadyInRoom, sequence).await;
            }
            let Some(next_generation) = state.generation.checked_add(1) else {
                return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
            };
            let binding = BindingKey {
                connection_id: state.connection_id,
                generation: next_generation,
            };
            let result = rooms
                .resume(
                    room_id,
                    binding,
                    sequence,
                    resume_token,
                    std::time::Instant::now(),
                )
                .await;
            admit(socket, result, binding, state, sequence).await
        }
        GameClientMessage::LeaveRoom {
            room_id, round_id, ..
        } => {
            leave(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::Ready {
            room_id,
            round_id,
            ready,
            ..
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Ready(ready),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::Movement {
            room_id,
            round_id,
            direction,
            ..
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Movement(direction.map(domain_direction)),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::PlaceBomb {
            room_id,
            round_id,
            action_id,
            ..
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Bomb(action_id.get()),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::Rematch {
            room_id,
            round_id,
            ready,
            ..
        } => {
            action(
                socket,
                rooms,
                room_id,
                u64::from(round_id.get()),
                RoomAction::Rematch(ready),
                state,
                sequence,
            )
            .await
        }
        GameClientMessage::Authenticate { .. } => unreachable!(),
    }
}

async fn admit(
    socket: &mut WebSocket,
    result: Result<Admission, RoomError>,
    binding: BindingKey,
    state: &mut ConnectionState,
    sequence: Counter,
) -> bool {
    match result {
        Ok(admission) => {
            debug_assert_eq!(admission.round_id, u64::from(admission.room.round_id.get()));
            let message = GameServerMessage::Admitted {
                sequence: admission.sequence,
                room: admission.room.clone(),
                you_seat: admission.seat,
                resume_token: admission.resume_token.clone(),
                last_bomb_action_id: admission.last_bomb_action_id,
                state: admission.state.clone(),
            };
            if send_game(socket, &message).await.is_err() {
                return false;
            }
            state.generation = binding.generation;
            state.membership = Some(Membership {
                admission,
                binding,
                replacement_closed: false,
            });
            true
        }
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn leave(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    state: &mut ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    if member.admission.room_id != room_id {
        return reply_error(socket, ErrorCode::StaleRoom, sequence).await;
    }
    let Some(next_generation) = state.generation.checked_add(1) else {
        return reply_error(socket, ErrorCode::CapacityReached, sequence).await;
    };
    match rooms
        .leave(&member.admission, member.binding, room_id.clone(), round_id)
        .await
    {
        Ok(()) => {
            let mut member = state.membership.take().expect("membership exists");
            member.admission.guard.disarm();
            state.generation = next_generation;
            state.deadlines.restart_unseated(Instant::now());
            send_game(socket, &GameServerMessage::LeftRoom { room_id, sequence })
                .await
                .is_ok()
        }
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn action(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    action: RoomAction,
    state: &ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    match rooms
        .action(&member.admission, member.binding, room_id, round_id, action)
        .await
    {
        Ok(()) => true,
        Err(error) => room_error(socket, error, sequence).await,
    }
}

async fn room_error(socket: &mut WebSocket, error: RoomError, sequence: Counter) -> bool {
    let (code, close_after) = match error {
        RoomError::RateLimited => (ErrorCode::RateLimited, true),
        RoomError::CapacityReached => (ErrorCode::CapacityReached, false),
        RoomError::RoomNotFound => (ErrorCode::RoomNotFound, false),
        RoomError::RoomFull => (ErrorCode::RoomFull, false),
        RoomError::StaleRoom => (ErrorCode::StaleRoom, false),
        RoomError::InvalidPhase => (ErrorCode::InvalidPhase, false),
        RoomError::DuplicateAction => (ErrorCode::DuplicateAction, false),
        RoomError::ResumeRejected => (ErrorCode::ResumeRejected, false),
    };
    if close_after {
        fail_and_close(socket, code, Some(sequence)).await;
        false
    } else {
        reply_error(socket, code, sequence).await
    }
}

const fn domain_direction(direction: WireDirection) -> grenadiers_core::game::Direction {
    match direction {
        WireDirection::North => grenadiers_core::game::Direction::North,
        WireDirection::East => grenadiers_core::game::Direction::East,
        WireDirection::South => grenadiers_core::game::Direction::South,
        WireDirection::West => grenadiers_core::game::Direction::West,
    }
}

async fn reply_error(socket: &mut WebSocket, code: ErrorCode, sequence: Counter) -> bool {
    send_error(socket, code, Some(sequence)).await.is_ok()
}
