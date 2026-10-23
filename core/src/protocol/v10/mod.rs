mod projection_admission;
mod projection_recipient;
mod scalars;
mod server;
mod state;

use std::fmt;

pub use crate::protocol::v8::{
    ARENA_TERRAIN_CELLS, ArenaView, BlastKind, BombTrigger, BombView, CompletedRound,
    ConnectionStatus, CountdownKind, DamageGraceTicks, DamagePreventionSource, EffectKind,
    EffectValidationError, EffectView, EffectViews, EquipmentId, EventIndex, EventKind, FlameView,
    GameClientMessage, GameSettings, HiddenActorView, InventoryView, ItemId, ItemMask, ItemSet,
    MAX_BLOCKS, MAX_BOMBS, MAX_FLAMES, MAX_PICKUPS, MAX_RUNNING_UPDATES, ManualActionKind, MapId,
    MatchInterruptReason, MatchResult, MatchWinnerReason, PickupKind, PickupView, ResumeToken,
    RoomMetadata, RoomPhase, RoomView, RoundCount, RulesVersion, Scores, SeatStatus,
    SettingsContextError, SnapshotEvents, Terrain, V8ErrorCode as V10ErrorCode, WeaponId, Wins,
    WireEvent, validate_effects, validate_settings_context,
};
pub use crate::protocol::v8::{BoundPublication, ExpandedCapture, ProjectionError};
pub use projection_admission::prepare_expanded_admission;
pub use projection_recipient::prepare_snapshot_pair;
pub use scalars::GameplayVersion;
pub use server::GameServerMessage;
pub use state::{ActorView, GameState, VisibleActorView};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 10;

pub fn capture_expanded(
    state: &crate::game::State,
    reports: &[crate::game::StepReport],
) -> Result<ExpandedCapture, ProjectionError> {
    crate::protocol::v8::projection_capture::capture_expanded_for_items(
        state,
        reports,
        expanded_items(),
    )
}

pub fn bind_expanded(
    room: &RoomView,
    state_epoch: crate::protocol::Counter,
    capture: &ExpandedCapture,
    generation: Option<crate::game::GenerationRecord>,
) -> Result<BoundPublication, ProjectionError> {
    crate::protocol::v8::projection_binding::bind_expanded_for_items(
        room,
        state_epoch,
        capture,
        generation,
        expanded_items(),
    )
}

pub fn expanded_settings(
    settings: GameSettings,
) -> Result<crate::room::RoomSettings, ProjectionError> {
    crate::protocol::v8::projection_room::expanded_settings_for_items(settings, expanded_items())
}

fn expanded_items() -> ItemSet {
    ItemSet::new([
        ItemId::WallPass,
        ItemId::Heart,
        ItemId::RemoteBomb,
        ItemId::PiercingBomb,
        ItemId::PowerBomb,
        ItemId::Landmine,
        ItemId::Shield,
        ItemId::Stopwatch,
        ItemId::InvisibilityCloak,
        ItemId::InvisibleBombs,
        ItemId::GoldenFlame,
        ItemId::ShortFuse,
        ItemId::Diarrhea,
        ItemId::SlowFoot,
        ItemId::ShortFlame,
        ItemId::LongFuse,
        ItemId::ReverseControls,
        ItemId::Teleporter,
    ])
    .expect("gameplay v10 expanded items are canonical")
}

pub fn project_expanded_room(
    metadata: RoomMetadata,
    lifecycle: &crate::room::MatchLifecycle,
) -> Result<RoomView, ProjectionError> {
    crate::protocol::v8::projection_room::project_expanded_room_for_items(
        metadata,
        lifecycle,
        expanded_items(),
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v10 client message")
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
#[cfg(test)]
mod wall_pass_tests;
