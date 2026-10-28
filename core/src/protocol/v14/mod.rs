mod client;
mod configuration;
mod events;
mod identity;
mod projection;
mod scalars;
mod server;
mod state;

use std::fmt;

pub use crate::protocol::v11::{
    ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks, EffectKind, EffectView,
    EffectViews, EquipmentId, FlameView, GameSettings, InventoryView, ItemId, ItemMask, ItemSet,
    OilBarrelState, OilBarrelView, PickupKind, PickupView, ResumeToken, RoundCount, Terrain,
    V11ErrorCode, WeaponId, Wins,
};
pub use client::{GameClientMessage, ManualActionKind};
pub use configuration::{BoardSize, ConfigurationController, Difficulty, RoomConfiguration};
pub use events::{
    DamagePreventionSource, EventIndex, EventKind, MAX_EVENT_INDEX, MAX_SNAPSHOT_EVENTS,
    SnapshotEvents, WireEvent,
};
pub use identity::{MapId, RulesVersion};
pub use projection::{ProjectionError, validate_room_view};
pub use scalars::{GameplayVersion, Seat};
pub use server::{
    CompletedRound, ConnectionStatus, Controller, CountdownKind, DrawReason, GameServerMessage,
    MatchInterruptReason, MatchResult, MatchWinnerReason, RoomPhase, RoomView, RoundResult, Score,
    Scores, SeatStatus,
};
pub use state::{ActorView, GameState, VisibleActorView};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 14;
pub const MAX_CLIENT_MESSAGE_BYTES: usize = 4_096;
pub const MAX_SERVER_MESSAGE_BYTES: usize = 98_304;
pub const MAX_BOMBS: usize = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v14 client message")
    }
}

impl std::error::Error for ParseClientMessageError {}

pub fn parse_client_message(input: &str) -> Result<GameClientMessage, ParseClientMessageError> {
    if input.len() > MAX_CLIENT_MESSAGE_BYTES {
        return Err(ParseClientMessageError::InvalidMessage);
    }
    serde_json::from_str(input).map_err(|_| ParseClientMessageError::InvalidMessage)
}

pub fn encode_server_message(
    message: &GameServerMessage,
) -> Result<String, super::EncodeServerMessageError> {
    let encoded = serde_json::to_string(message)
        .map_err(|_| super::EncodeServerMessageError::EncodingFailed)?;
    enforce_server_message_size(encoded)
}

fn enforce_server_message_size(encoded: String) -> Result<String, super::EncodeServerMessageError> {
    if encoded.len() > MAX_SERVER_MESSAGE_BYTES {
        Err(super::EncodeServerMessageError::MessageTooLarge)
    } else {
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests;
