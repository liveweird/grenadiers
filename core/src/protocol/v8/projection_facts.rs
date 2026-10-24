use std::collections::BTreeMap;

use crate::game::{EventAudience, RulesConfig};
use crate::protocol::{Cell, Direction, EntityId, Tick, WireRoundResult};

use super::{
    ArenaView, BombView, EventKind, FlameView, ItemSet, PickupView, RoomView, VisibleActorView,
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct WorldFacts {
    pub(crate) rules_version: super::RulesVersion,
    pub(crate) physical_tick: Tick,
    pub(crate) last_teleport_tick: Option<Tick>,
    pub(crate) running_updates: u16,
    pub(crate) remaining_ticks: u16,
    pub(crate) result: Option<WireRoundResult>,
    pub(crate) arena: ArenaView,
    pub(crate) actors: [VisibleActorView; 2],
    pub(crate) last_successful_punch_ticks: [Option<Tick>; 2],
    pub(crate) last_successful_shove_ticks: [Option<Tick>; 2],
    pub(crate) soft_wall_grace: [Vec<Cell>; 2],
    pub(crate) unlit_oil_grace: [Vec<Cell>; 2],
    pub(crate) bombs: Vec<BombView>,
    pub(crate) timed_bombs: BTreeMap<EntityId, TimedBombFacts>,
    pub(crate) kick_bombs: BTreeMap<EntityId, KickBombFacts>,
    pub(crate) punch_bombs: BTreeMap<EntityId, PunchBombFacts>,
    pub(crate) flames: Vec<FlameView>,
    pub(crate) pickups: Vec<PickupView>,
    pub(crate) private_pickup_cells: Vec<Cell>,
    pub(crate) oil: Vec<OilFacts>,
    pub(crate) available_items: ItemSet,
    pub(crate) enabled_items: ItemSet,
}

impl std::fmt::Debug for WorldFacts {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("WorldFacts(<redacted>)")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OilFacts {
    pub(crate) cell: Cell,
    pub(crate) phase: OilPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OilPhase {
    Concealed,
    Pending,
    Unlit,
    Burning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EventFacts {
    pub(crate) tick: Tick,
    pub(crate) audience: EventAudience,
    pub(crate) kind: EventKind,
    pub(crate) timed_bomb: Option<TimedBombFacts>,
    pub(crate) kick_bomb: Option<KickBombFacts>,
    pub(crate) punch_bomb: Option<PunchBombFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimedBombFacts {
    pub(crate) captured_fuse_ticks: u64,
    pub(crate) paused_ticks: u64,
    pub(crate) detonates_at: Tick,
    pub(crate) paused: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BombMotionFacts {
    pub(crate) direction: Direction,
    pub(crate) countdown: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KickBombFacts {
    pub(crate) bomb: BombView,
    pub(crate) motion: Option<BombMotionFacts>,
    pub(crate) has_been_kicked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PunchBombFacts {
    pub(crate) bomb: BombView,
    pub(crate) has_been_punched: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReportFacts {
    pub(crate) tick: Tick,
    pub(crate) events: Vec<EventFacts>,
    pub(crate) forced_movement: Option<ForcedMovementFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForcedMovementFacts {
    pub(crate) victim: crate::protocol::Seat,
    pub(crate) start: crate::protocol::Position,
    pub(crate) end: crate::protocol::Position,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedWorld(pub(crate) WorldFacts);

#[derive(Clone, PartialEq, Eq)]
pub struct ValidatedReports(pub(crate) Vec<ReportFacts>);

#[derive(Clone, PartialEq, Eq)]
pub struct ExpandedCapture {
    pub(crate) rules: RulesConfig,
    pub(crate) world: ValidatedWorld,
    pub(crate) reports: ValidatedReports,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BoundPublication {
    pub(crate) room: RoomView,
    pub(crate) world: ValidatedWorld,
    pub(crate) reports: ValidatedReports,
}

impl std::fmt::Debug for ValidatedWorld {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ValidatedWorld(<redacted>)")
    }
}

impl std::fmt::Debug for ValidatedReports {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ValidatedReports(<redacted>)")
    }
}

impl std::fmt::Debug for ExpandedCapture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ExpandedCapture")
            .field("rules", &self.rules)
            .field("world", &"<redacted>")
            .field("reports", &"<redacted>")
            .finish()
    }
}

impl std::fmt::Debug for BoundPublication {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("BoundPublication")
            .field("room", &self.room)
            .field("world", &"<redacted>")
            .field("reports", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionError {
    InvalidRules,
    InvalidClock,
    InvalidArena,
    InvalidActors,
    InvalidBombs,
    InvalidFlames,
    InvalidPickups,
    InvalidResult,
    InvalidEffects,
    InvalidItems,
    InvalidReports,
    InvalidEvent,
    InvalidClassicBinding,
    InvalidExpandedBinding,
    EncodingFailed,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for ProjectionError {}
