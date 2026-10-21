use std::collections::BTreeMap;

use crate::game::{EventAudience, RulesConfig};
use crate::protocol::{EntityId, Tick, WireRoundResult};

use super::{
    ArenaView, BombView, EventKind, FlameView, ItemSet, PickupView, RoomView, VisibleActorView,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct WorldFacts {
    pub(crate) rules_version: super::RulesVersion,
    pub(crate) physical_tick: Tick,
    pub(crate) running_updates: u16,
    pub(crate) remaining_ticks: u16,
    pub(crate) result: Option<WireRoundResult>,
    pub(crate) arena: ArenaView,
    pub(crate) actors: [VisibleActorView; 2],
    pub(crate) bombs: Vec<BombView>,
    pub(crate) timed_bombs: BTreeMap<EntityId, TimedBombFacts>,
    pub(crate) flames: Vec<FlameView>,
    pub(crate) pickups: Vec<PickupView>,
    pub(crate) available_items: ItemSet,
    pub(crate) enabled_items: ItemSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EventFacts {
    pub(crate) tick: Tick,
    pub(crate) audience: EventAudience,
    pub(crate) kind: EventKind,
    pub(crate) timed_bomb: Option<TimedBombFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimedBombFacts {
    pub(crate) captured_fuse_ticks: u64,
    pub(crate) paused_ticks: u64,
    pub(crate) detonates_at: Tick,
    pub(crate) paused: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReportFacts {
    pub(crate) tick: Tick,
    pub(crate) events: Vec<EventFacts>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedWorld(pub(crate) WorldFacts);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedReports(pub(crate) Vec<ReportFacts>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExpandedCapture {
    pub(crate) rules: RulesConfig,
    pub(crate) world: ValidatedWorld,
    pub(crate) reports: ValidatedReports,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundPublication {
    pub(crate) room: RoomView,
    pub(crate) world: ValidatedWorld,
    pub(crate) reports: ValidatedReports,
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
