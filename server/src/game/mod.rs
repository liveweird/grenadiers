mod connection;
mod dispatcher;
mod http;
mod limits;
mod rooms;
mod sessions;
mod socket;

pub(crate) use http::issue_session;
pub(crate) use rooms::GameRooms;
pub(crate) use sessions::GameSessions;
pub(crate) use socket::game_websocket;

#[cfg(test)]
#[path = "room_socket_tests.rs"]
mod room_socket_tests;
