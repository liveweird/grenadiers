use super::{GameState, ResumeToken, RoomConfiguration, RoomSettings, Seat, SnapshotEvents};
use crate::protocol::{Counter, ProtocolVersionInput, RoomCode, RoomId, Tick};
use serde::Serialize;

use crate::protocol::v14::{CompletedRound, ErrorCode, Scores};
use crate::protocol::v19::{RulesVersion, SeatStatus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GameplayVersion;
impl Serialize for GameplayVersion {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(super::GAMEPLAY_PROTOCOL_VERSION)
    }
}

pub use crate::protocol::v16::RoomPhase;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RoomView {
    pub settings: RoomSettings,
    pub settings_revision: Counter,
    pub settings_editor: Option<Seat>,
    pub available_items: super::ItemSet,
    pub configuration: RoomConfiguration,
    #[serde(rename = "roomId")]
    pub room_id: RoomId,
    #[serde(rename = "roomCode")]
    pub room_code: RoomCode,
    #[serde(rename = "roundId")]
    pub round_id: Counter,
    pub revision: Counter,
    pub tick: Tick,
    #[serde(rename = "rulesVersion")]
    pub rules_version: RulesVersion,
    #[serde(rename = "mapId")]
    pub map_id: super::MapId,
    #[serde(rename = "roundNumber")]
    pub round_number: super::RoundCount,
    pub scores: Scores,
    #[serde(rename = "lastRound")]
    pub last_round: Option<CompletedRound>,
    pub seats: Vec<SeatStatus>,
    pub phase: RoomPhase,
}

impl RoomView {
    #[must_use]
    pub fn from_v20(room: crate::protocol::v20::RoomView, settings: RoomSettings) -> Self {
        Self {
            settings,
            settings_revision: room.settings_revision,
            settings_editor: room.settings_editor,
            available_items: room.available_items,
            configuration: room.configuration,
            room_id: room.room_id,
            room_code: room.room_code,
            round_id: room.round_id,
            revision: room.revision,
            tick: room.tick,
            rules_version: room.rules_version,
            map_id: room.map_id,
            round_number: room.round_number,
            scores: room.scores,
            last_round: room.last_round,
            seats: room.seats,
            phase: room.phase,
        }
    }

    pub(crate) fn legacy_view(&self) -> crate::protocol::v20::RoomView {
        crate::protocol::v20::RoomView {
            settings: crate::protocol::v20::RoomSettings::new(
                self.settings.base,
                self.settings.environment,
            ),
            settings_revision: self.settings_revision,
            settings_editor: self.settings_editor,
            available_items: self.available_items,
            configuration: self.configuration.clone(),
            room_id: self.room_id.clone(),
            room_code: self.room_code.clone(),
            round_id: self.round_id,
            revision: self.revision,
            tick: self.tick,
            rules_version: self.rules_version,
            map_id: self.map_id,
            round_number: self.round_number,
            scores: self.scores.clone(),
            last_round: self.last_round,
            seats: self.seats.clone(),
            phase: self.phase.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum GameServerMessage {
    Authenticated {
        #[serde(rename = "protocolVersion")]
        protocol_version: GameplayVersion,
    },
    IncompatibleProtocol {
        #[serde(rename = "supportedVersion")]
        supported_version: ProtocolVersionInput,
    },
    Admitted {
        sequence: Counter,
        room: RoomView,
        #[serde(rename = "youSeat")]
        you_seat: Seat,
        #[serde(rename = "resumeToken")]
        resume_token: ResumeToken,
        #[serde(rename = "lastActionId")]
        last_action_id: Option<Counter>,
        state: Option<GameState>,
    },
    RoomState {
        room: RoomView,
    },
    Snapshot {
        room: RoomView,
        state: GameState,
        events: SnapshotEvents,
    },
    LeftRoom {
        #[serde(rename = "roomId")]
        room_id: RoomId,
        sequence: Counter,
    },
    SeatReplaced {
        #[serde(rename = "roomId")]
        room_id: RoomId,
    },
    Error {
        code: ErrorCode,
        sequence: Option<Counter>,
    },
}
