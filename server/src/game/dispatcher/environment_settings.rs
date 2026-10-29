use axum::extract::ws::WebSocket;
use grenadiers_core::protocol::{Counter, RoomId, v15::ErrorCode};

use super::{reply_error, room_error};
use crate::game::{
    connection::ConnectionState,
    rooms::{EnvironmentSettingsAction, GameRooms},
};

pub(super) async fn dispatch(
    socket: &mut WebSocket,
    rooms: &GameRooms,
    room_id: RoomId,
    round_id: u64,
    action: EnvironmentSettingsAction,
    state: &ConnectionState,
    sequence: Counter,
) -> bool {
    let Some(member) = state.membership.as_ref() else {
        return reply_error(socket, ErrorCode::NotInRoom, sequence).await;
    };
    match rooms
        .environment_settings_live(&member.admission, member.binding, room_id, round_id, action)
        .await
    {
        Ok(()) => true,
        Err(error) => room_error(socket, error, sequence).await,
    }
}
