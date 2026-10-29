mod client;
mod configuration;
mod cpu_observation;
mod cpu_observation_types;
mod events;
mod identity;
mod projection;
mod projection_actor_validation;
mod projection_admission;
mod projection_binding;
mod projection_bomb_validation;
pub(crate) mod projection_capture;
mod projection_capture_wire;
mod projection_event_validation;
pub(crate) mod projection_facts;
mod projection_geometry;
mod projection_history;
mod projection_item_validation;
pub(crate) mod projection_lifecycle_facts;
mod projection_oil;
pub(crate) mod projection_recipient;
pub(crate) mod projection_room_validation;
pub(crate) mod projection_supported_items;
pub(crate) mod projection_validation;
mod projection_world_validation;
mod scalars;
mod server;
mod state;

use std::fmt;

pub use crate::protocol::v11::{
    ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks, EffectKind, EffectView,
    EffectViews, EquipmentId, FlameView, GameSettings, InventoryView, ItemId, ItemMask, ItemSet,
    OilBarrelState, OilBarrelView, PickupKind, PickupView, ResumeToken, RoundCount, Terrain,
    WeaponId, Wins, validate_effects,
};
pub use client::{GameClientMessage, ManualActionKind};
pub use configuration::{BoardSize, ConfigurationController, Difficulty, RoomConfiguration};
pub use cpu_observation::observe_cpu_recipient;
pub(crate) use cpu_observation::{observation_error, observation_parts};
pub use events::{
    DamagePreventionSource, EventIndex, EventKind, MAX_EVENT_INDEX, MAX_SNAPSHOT_EVENTS,
    SnapshotEvents, WireEvent,
};
pub use identity::{MapId, RulesVersion};
pub use projection::{ProjectionError, validate_room_view};
pub use projection_admission::{AdmissionProjection, PersonalAdmission, prepare_admission};
pub use projection_binding::bind_expanded;
pub use projection_capture::capture_expanded;
pub use projection_facts::{BoundPublication, ExpandedCapture};
pub use projection_recipient::{RecipientProjection, prepare_snapshots, project_recipient};
pub use scalars::{GameplayVersion, Seat};
pub use server::{
    CompletedRound, ConnectionStatus, Controller, CountdownKind, DrawReason, ErrorCode,
    GameServerMessage, MatchInterruptReason, MatchResult, MatchWinnerReason, RoomPhase, RoomView,
    RoundResult, Score, Scores, SeatStatus,
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
mod capture_tests;
#[cfg(test)]
mod cpu_observation_tests;
#[cfg(test)]
mod frame_bound_tests;
#[cfg(test)]
mod max_frame_tests;
#[cfg(test)]
mod oil_validation_tests;
#[cfg(test)]
mod projection_certification_tests;
#[cfg(test)]
mod projection_private_facts_tests;
#[cfg(test)]
mod projection_validation_tests;
#[cfg(test)]
mod room_validation_tests;
#[cfg(test)]
mod teleport_privacy_tests;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod abandonment_tests;

#[cfg(test)]
mod board_geometry_tests;
