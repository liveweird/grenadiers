mod client;
mod identity;
mod projection_admission;
mod projection_binding;
mod projection_capture;
mod projection_facts;
mod projection_items;
mod projection_recipient;
mod projection_room;
mod projection_room_settings;
mod projection_room_validation;
mod projection_validation;
mod scalars;
mod server;
mod settings;

use std::fmt;

pub(crate) use crate::protocol::v7::BombTriggerObservation;
pub use crate::protocol::v7::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks,
    DamagePreventionSource, EffectKind, EffectValidationError, EffectView, EffectViews,
    EquipmentId, EventIndex, EventKind, FlameView, GameState, HiddenActorView, InventoryView,
    ItemId, ItemMask, ItemSet, MAX_BLOCKS, MAX_BOMBS, MAX_FLAMES, MAX_PICKUPS, MAX_RUNNING_UPDATES,
    PickupKind, PickupView, ResumeToken, RoundCount, SnapshotEvents, Terrain, VisibleActorView,
    WeaponId, Wins, WireEvent, validate_effects,
};
pub use client::{GameClientMessage, ManualActionKind};
pub use identity::{MapId, RulesVersion};
pub use projection_admission::prepare_expanded_admission;
pub use projection_binding::bind_expanded;
pub use projection_capture::capture_expanded;
pub use projection_facts::{
    BoundPublication, ExpandedCapture, ProjectionError, ValidatedReports, ValidatedWorld,
};
pub use projection_recipient::prepare_snapshot_pair;
pub use projection_room::{RoomMetadata, expanded_settings, project_expanded_room};
pub use scalars::GameplayVersion;
pub use server::{
    CompletedRound, ConnectionStatus, CountdownKind, GameServerMessage, MatchInterruptReason,
    MatchResult, MatchWinnerReason, RoomPhase, RoomView, Scores, SeatStatus, V8ErrorCode,
};
pub use settings::{GameSettings, SettingsContextError, validate_settings_context};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v8 client message")
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

#[cfg(test)]
#[path = "frame_bound_tests.rs"]
mod frame_bound_tests;
#[cfg(test)]
#[path = "max_frame_tests.rs"]
mod max_frame_tests;
#[cfg(test)]
mod tests;
