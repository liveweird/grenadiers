use axum::extract::ws::WebSocket;
use grenadiers_core::protocol::{Counter, RoomId, v18::ErrorCode};

use super::{reply_error, room_error};
use crate::game::{
    connection::ConnectionState,
    rooms::{GameRooms, LairSettingsAction},
};

pub(super) async fn dispatch(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    action: LairSettingsAction,
    state: &ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    match rooms
        .lair_settings_live(&member.admission, member.binding, room_id, round_id, action)
        .await
    {
        Ok(()) => true,
        Err(error) => room_error(socket, error, sequence).await,
    }
}

pub(super) const fn domain_direction(
    direction: grenadiers_core::protocol::Direction,
) -> grenadiers_core::game::Direction {
    match direction {
        grenadiers_core::protocol::Direction::North => grenadiers_core::game::Direction::North,
        grenadiers_core::protocol::Direction::East => grenadiers_core::game::Direction::East,
        grenadiers_core::protocol::Direction::South => grenadiers_core::game::Direction::South,
        grenadiers_core::protocol::Direction::West => grenadiers_core::game::Direction::West,
    }
}
