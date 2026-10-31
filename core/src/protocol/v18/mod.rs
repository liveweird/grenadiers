mod admission;
mod client;
mod configuration;
mod cpu_observation;
mod projection;
mod projection_validation;
mod room_validation;
mod server;
mod settings;
mod state;

use std::fmt;

pub use crate::protocol::v14::ProjectionError;
pub use crate::protocol::v14::{
    ArenaView, BlastKind, BoardSize, BombTrigger, BombView, ConfigurationController,
    DamageGraceTicks, Difficulty, EffectKind, EffectView, EffectViews, EquipmentId, FlameView,
    GameSettings, InventoryView, ItemId, ItemMask, ItemSet, MapId, OilBarrelState, OilBarrelView,
    PickupKind, PickupView, ResumeToken, RoundCount, Seat, SnapshotEvents, Terrain, WeaponId, Wins,
    WireEvent,
};
pub use admission::{AdmissionProjection, PersonalAdmission, prepare_admission};
pub use client::{GameClientMessage, ManualActionKind};
pub use configuration::{Environment, Lairs, RoomConfiguration, SpikePits};
pub use cpu_observation::observe_cpu_recipient;
pub use projection::{
    BoundPublication, ExpandedCapture, RecipientProjection, bind_expanded, capture_expanded,
    prepare_snapshots, project_recipient,
};
pub use room_validation::validate_room_view;
pub use server::{
    CompletedRound, ConnectionStatus, Controller, CountdownKind, DrawReason, ErrorCode,
    GameServerMessage, GameplayVersion, MatchInterruptReason, MatchResult, MatchWinnerReason,
    RoomPhase, RoomView, RoundResult, RulesVersion, Score, Scores, SeatStatus,
};
pub use settings::RoomSettings;
pub use state::{
    ActorView, GameState, LairView, MobView, SpikePitPhase, SpikePitView, StormCloudView,
};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 18;
pub const MAX_CLIENT_MESSAGE_BYTES: usize = 4_096;
pub const MAX_SERVER_MESSAGE_BYTES: usize = 196_608;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}
impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid gameplay v18 client message")
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
mod mutable_world_tests;
#[cfg(test)]
mod server_conformance_tests;
#[cfg(test)]
mod tests;
