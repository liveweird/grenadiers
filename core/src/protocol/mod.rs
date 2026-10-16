mod client;
mod events;
mod scalars;
mod server;
mod snapshot;

use std::fmt;

pub use client::GameClientMessage;
pub use events::{EventKind, MAX_EVENT_INDEX, MAX_SNAPSHOT_EVENTS, WireEvent};
pub use scalars::{
    Cell, Counter, Direction, EntityId, GAMEPLAY_PROTOCOL_VERSION, GameplayVersion, InvalidScalar,
    MAX_COUNTER, MAX_SAFE_TICK, MapId, Position, ProtocolVersionInput, RULES_VERSION, RoomCode,
    RoomId, RulesVersion, Seat, SessionToken, Tick,
};
pub use server::{
    DrawReason, ErrorCode, GameServerMessage, InterruptReason, RoomPhase, RoomView, SeatStatus,
    WireRoundResult,
};
pub use snapshot::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BombView, FlameView, GameState, MAX_BLOCKS,
    MAX_BOMBS, MAX_FLAMES, MAX_RUNNING_UPDATES, Terrain,
};

pub const MAX_CLIENT_MESSAGE_BYTES: usize = 4_096;
pub const MAX_SERVER_MESSAGE_BYTES: usize = 65_536;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay client message")
    }
}

impl std::error::Error for ParseClientMessageError {}

pub fn parse_client_message(input: &str) -> Result<GameClientMessage, ParseClientMessageError> {
    if input.len() > MAX_CLIENT_MESSAGE_BYTES {
        return Err(ParseClientMessageError::InvalidMessage);
    }
    serde_json::from_str(input).map_err(|_| ParseClientMessageError::InvalidMessage)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EncodeServerMessageError {
    EncodingFailed,
    MessageTooLarge,
}

impl fmt::Display for EncodeServerMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EncodingFailed => formatter.write_str("gameplay message encoding failed"),
            Self::MessageTooLarge => formatter.write_str("gameplay server message is too large"),
        }
    }
}

impl std::error::Error for EncodeServerMessageError {}

pub fn encode_server_message(
    message: &GameServerMessage,
) -> Result<String, EncodeServerMessageError> {
    let encoded =
        serde_json::to_string(message).map_err(|_| EncodeServerMessageError::EncodingFailed)?;
    enforce_server_message_size(encoded)
}

fn enforce_server_message_size(encoded: String) -> Result<String, EncodeServerMessageError> {
    if encoded.len() > MAX_SERVER_MESSAGE_BYTES {
        Err(EncodeServerMessageError::MessageTooLarge)
    } else {
        Ok(encoded)
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod client_tests;
#[cfg(test)]
#[path = "server_tests.rs"]
mod server_tests;
#[cfg(test)]
#[path = "snapshot_tests.rs"]
mod snapshot_tests;
