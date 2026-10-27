mod client;
mod projection;
mod projection_admission;
mod projection_recipient;
mod scalars;
mod server;

use std::fmt;

pub use crate::protocol::v11::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BlastKind, BombTrigger, BombView, CompletedRound,
    CountdownKind, DamageGraceTicks, DamagePreventionSource, EffectKind, EffectValidationError,
    EffectView, EffectViews, EquipmentId, EventIndex, EventKind, ExpandedCapture, FlameView,
    GameSettings, GameState, HiddenActorView, InventoryView, ItemId, ItemMask, ItemSet, MAX_BLOCKS,
    MAX_BOMBS, MAX_FLAMES, MAX_PICKUPS, MAX_RUNNING_UPDATES, MapId, MatchInterruptReason,
    MatchResult, MatchWinnerReason, OilBarrelState, OilBarrelView, PickupKind, PickupView,
    ProjectionError, ResumeToken, RoomPhase, RoundCount, RulesVersion, Scores,
    SettingsContextError, SnapshotEvents, Terrain, V11ErrorCode, VisibleActorView, Wins, WireEvent,
    capture_expanded, expanded_settings, validate_effects, validate_settings_context,
};
pub use client::{GameClientMessage, ManualActionKind};
pub use projection::{BoundPublication, bind_expanded, project_expanded_room, validate_room_view};
pub use projection_admission::{AdmissionProjection, PersonalAdmission, prepare_admission};
pub use projection_recipient::prepare_snapshot_pair;
pub use server::{
    ConnectionStatus, Controller, Difficulty, GameServerMessage, RoomMode, RoomView, SeatStatus,
};

pub use scalars::GameplayVersion;

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 13;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomMetadata {
    pub settings: GameSettings,
    pub settings_revision: crate::protocol::Counter,
    pub settings_editor: Option<crate::protocol::Seat>,
    pub room_id: crate::protocol::RoomId,
    pub room_code: crate::protocol::RoomCode,
    pub revision: crate::protocol::Counter,
    pub rules_version: RulesVersion,
    pub map_id: MapId,
    pub mode: crate::room::RoomMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v13 client message")
    }
}

impl std::error::Error for ParseClientMessageError {}

pub fn parse_client_message(input: &str) -> Result<GameClientMessage, ParseClientMessageError> {
    if input.len() > super::MAX_CLIENT_MESSAGE_BYTES {
        return Err(ParseClientMessageError::InvalidMessage);
    }
    serde_json::from_str(input).map_err(|_| ParseClientMessageError::InvalidMessage)
}

pub fn encode_server_message(
    message: &GameServerMessage,
) -> Result<String, super::EncodeServerMessageError> {
    let encoded = serde_json::to_string(message)
        .map_err(|_| super::EncodeServerMessageError::EncodingFailed)?;
    super::enforce_server_message_size(encoded)
}

/// Derives the CPU's neutral observation from the same checked, recipient-private projection.
pub fn observe_cpu_recipient(
    publication: &BoundPublication,
    recipient: crate::protocol::Seat,
    world: crate::cpu::WorldId,
) -> Result<crate::cpu::Observation, ProjectionError> {
    crate::protocol::v11::observe_cpu_recipient(&publication.physics, recipient, world)
}

#[cfg(test)]
mod tests;
