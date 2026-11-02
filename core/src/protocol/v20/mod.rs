mod admission;
mod client;
mod projection;
mod server;

use std::fmt;

pub use crate::protocol::v19::validate_room_view;
pub use crate::protocol::v19::{
    ActorView, ArenaView, BlastKind, BoardSize, BombTrigger, BombView, CompletedRound,
    ConfigurationController, ConnectionStatus, Controller, CountdownKind, DamageGraceTicks,
    Difficulty, DrawReason, EffectKind, EffectView, EffectViews, Environment, EquipmentId,
    ErrorCode, FlameView, GameSettings, GameState, InventoryView, ItemId, ItemMask, ItemSet,
    LairView, Lairs, MapId, MatchInterruptReason, MatchResult, MatchWinnerReason, MobView,
    OilBarrelState, OilBarrelView, PickupKind, PickupView, ProjectionError, ResumeToken,
    RoomConfiguration, RoomPhase, RoomSettings, RoundCount, RoundResult, RulesVersion, Score,
    Scores, Seat, SeatStatus, SnapshotEvents, SpikePitPhase, SpikePitView, SpikePits,
    StormCloudView, Terrain, WeaponId, Wins, WireEvent,
};
pub use admission::{AdmissionProjection, PersonalAdmission, prepare_admission};
pub use client::{GameClientMessage, ManualActionKind};
pub use projection::{
    BoundPublication, ExpandedCapture, RecipientProjection, bind_expanded, capture_expanded,
    prepare_snapshots, project_recipient,
};

pub fn observe_cpu_recipient(
    publication: &BoundPublication,
    recipient: Seat,
    world: crate::cpu::WorldId,
) -> Result<crate::cpu::Observation, ProjectionError> {
    crate::protocol::v19::observe_cpu_recipient_with_punch_activation(
        publication,
        recipient,
        world,
        crate::game::PunchActivationPolicy::ContactV2,
    )
}
pub use server::{GameServerMessage, GameplayVersion, RoomView};

pub const GAMEPLAY_PROTOCOL_VERSION: u16 = 20;
pub const MAX_CLIENT_MESSAGE_BYTES: usize = 4_096;
pub const MAX_SERVER_MESSAGE_BYTES: usize = 196_608;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseClientMessageError {
    InvalidMessage,
}

impl fmt::Display for ParseClientMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid gameplay v20 client message")
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
mod tests;
