mod client;
mod effects;
mod events;
mod items;
mod projection_admission;
mod projection_binding;
mod projection_capture;
mod projection_facts;
mod projection_items;
mod projection_recipient;
mod projection_room;
mod projection_validation;
mod scalars;
mod server;
mod settings;
mod state;

use std::fmt;

pub use client::{GameClientMessage, ManualActionKind};
pub use effects::{EffectKind, EffectValidationError, EffectView, EffectViews, validate_effects};
pub use events::{DamagePreventionSource, EventIndex, EventKind, SnapshotEvents, WireEvent};
pub use items::{EquipmentId, InventoryView, ItemId, ItemMask, ItemSet, PickupKind, WeaponId};
pub use projection_admission::prepare_classic_admission;
pub use projection_binding::bind_classic;
pub use projection_capture::capture_classic;
pub use projection_facts::{
    BoundPublication, ClassicCapture, ProjectionError, ValidatedReports, ValidatedWorld,
};
pub use projection_recipient::prepare_snapshot_pair;
pub use projection_room::{RoomMetadata, classic_settings, project_classic_room};
pub(crate) use scalars::deserialize_bounded;
pub use scalars::{GameplayVersion, ResumeToken, RoundCount, Wins};
pub use server::{
    CompletedRound, ConnectionStatus, CountdownKind, GameServerMessage, MatchInterruptReason,
    MatchResult, MatchWinnerReason, RoomPhase, RoomView, Scores, SeatStatus, V7ErrorCode,
};
pub use settings::{GameSettings, SettingsContextError, validate_settings_context};
pub(crate) use state::BombTriggerObservation;
pub use state::{
    ARENA_TERRAIN_CELLS, ActorView, ArenaView, BlastKind, BombTrigger, BombView, DamageGraceTicks,
    FlameView, GameState, HiddenActorView, MAX_BLOCKS, MAX_BOMBS, MAX_FLAMES, MAX_PICKUPS,
    MAX_RUNNING_UPDATES, PickupView, Terrain, VisibleActorView,
};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v7 client message")
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
#[path = "contract_tests.rs"]
mod contract_tests;
#[cfg(test)]
#[path = "frame_bound_tests.rs"]
mod frame_bound_tests;
#[cfg(test)]
#[path = "max_frame_tests.rs"]
mod max_frame_tests;
#[cfg(test)]
mod projection_admission_tests;
#[cfg(test)]
mod projection_elimination_tests;
#[cfg(test)]
mod projection_event_tests;
#[cfg(test)]
#[path = "projection_test_support.rs"]
mod projection_test_support;
#[cfg(test)]
#[path = "projection_tests.rs"]
mod projection_tests;
#[cfg(test)]
#[path = "projection_trigger_tests.rs"]
mod projection_trigger_tests;
