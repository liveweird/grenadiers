mod admission;
mod client;
mod configuration;
mod projection;
mod server;
mod settings;
mod state;

use std::fmt;

pub use crate::protocol::v14::{
    ArenaView, BlastKind, BoardSize, BombTrigger, BombView, ConfigurationController,
    DamageGraceTicks, Difficulty, EffectKind, EffectView, EffectViews, EquipmentId, FlameView,
    GameSettings, InventoryView, ItemId, ItemMask, ItemSet, MapId, OilBarrelState, OilBarrelView,
    PickupKind, PickupView, ResumeToken, RoundCount, Seat, SnapshotEvents, Terrain, WeaponId, Wins,
    WireEvent,
};
pub use admission::{AdmissionProjection, PersonalAdmission, prepare_admission};
pub use client::{GameClientMessage, ManualActionKind};
pub use configuration::{Environment, RoomConfiguration};
pub use projection::{
    BoundPublication, ExpandedCapture, RecipientProjection, bind_expanded, capture_expanded,
    prepare_snapshots, project_recipient,
};
pub use server::{
    CompletedRound, ConnectionStatus, Controller, CountdownKind, DrawReason, ErrorCode,
    GameServerMessage, GameplayVersion, MatchInterruptReason, MatchResult, MatchWinnerReason,
    RoomPhase, RoomView, RoundResult, RulesVersion, Score, Scores, SeatStatus,
};
pub use settings::RoomSettings;
pub use state::{ActorView, GameState, StormCloudView};

pub use crate::protocol::v14::ProjectionError;

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 15;
pub const MAX_CLIENT_MESSAGE_BYTES: usize = 4_096;
pub const MAX_SERVER_MESSAGE_BYTES: usize = 98_304;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}
impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v15 client message")
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
) -> Result<String, crate::protocol::EncodeServerMessageError> {
    let encoded = serde_json::to_string(message)
        .map_err(|_| crate::protocol::EncodeServerMessageError::EncodingFailed)?;
    if encoded.len() > MAX_SERVER_MESSAGE_BYTES {
        Err(crate::protocol::EncodeServerMessageError::MessageTooLarge)
    } else {
        Ok(encoded)
    }
}

#[cfg(test)]
mod server_conformance_lifecycle;
#[cfg(test)]
mod server_conformance_tests;
#[cfg(test)]
mod tests;
