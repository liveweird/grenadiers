use crate::game::{EventAudience, GenerationRecord, RulesConfig};
use crate::protocol::{Cell, Counter, Direction, Position, Tick};

use super::{
    ArenaView, BombView, EventKind, FlameView, ItemSet, PickupKind, PickupView, RoomView, Seat,
    VisibleActorView,
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct ActorFacts {
    pub view: VisibleActorView,
    pub damage_grace_received_at: Option<Tick>,
    pub last_teleport_tick: Option<Tick>,
    pub last_successful_punch_tick: Option<Tick>,
    pub last_successful_shove_tick: Option<Tick>,
    pub soft_wall_grace: Vec<Cell>,
    pub unlit_oil_grace: Vec<Cell>,
    pub storm_slow_remaining_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct StormCloudFacts {
    pub position: Position,
    pub direction: Direction,
    pub leg_remaining_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OilPhase {
    Concealed,
    Pending,
    Unlit,
    Burning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct OilFacts {
    pub cell: Cell,
    pub phase: OilPhase,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PrivatePickupPhase {
    Concealed,
    Pending,
    Exposed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PrivatePickupFacts {
    pub cell: Cell,
    pub kind: PickupKind,
    pub phase: PrivatePickupPhase,
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct WorldFacts {
    pub rules_version: u8,
    pub physical_tick: Tick,
    pub last_teleport_tick: Option<Tick>,
    pub running_updates: u16,
    pub remaining_ticks: u16,
    pub result: Option<super::RoundResult>,
    pub arena: ArenaView,
    pub actors: Vec<ActorFacts>,
    pub bombs: Vec<BombView>,
    pub bomb_history: Vec<BombHistory>,
    pub flames: Vec<FlameView>,
    pub pickups: Vec<PickupView>,
    pub private_pickup_cells: Vec<Cell>,
    pub private_pickups: Vec<PrivatePickupFacts>,
    pub oil: Vec<OilFacts>,
    pub available_items: ItemSet,
    pub enabled_items: ItemSet,
    pub storm_cloud: Option<StormCloudFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TriggerHistory {
    Timed {
        captured_fuse_ticks: u64,
        paused_ticks: u64,
        detonates_at: Tick,
        paused: bool,
    },
    Remote,
    Contact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BombHistory {
    pub id: crate::protocol::EntityId,
    pub trigger: TriggerHistory,
    pub motion: Option<(crate::protocol::Direction, u8)>,
    pub has_been_kicked: bool,
    pub has_been_punched: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EventFacts {
    pub tick: Tick,
    pub audience: EventAudience,
    pub kind: EventKind,
    pub bomb_history: Option<BombHistory>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ForcedMovementFacts {
    pub pusher: Seat,
    pub victim: Seat,
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReportFacts {
    pub tick: Tick,
    pub trace_count: usize,
    pub events: Vec<EventFacts>,
    pub forced_movements: Vec<ForcedMovementFacts>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ExpandedCapture {
    pub(crate) rules: RulesConfig,
    pub(crate) world: WorldFacts,
    pub(crate) reports: Vec<ReportFacts>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ProjectionProfile {
    pub rules: RulesConfig,
    pub available_items: ItemSet,
    pub storm_cloud: crate::game::StormCloudConfig,
}

#[derive(Clone, PartialEq, Eq)]
pub struct BoundPublication {
    pub(crate) room: RoomView,
    pub(crate) state_epoch: Counter,
    pub(crate) world: WorldFacts,
    pub(crate) reports: Vec<ReportFacts>,
    pub(crate) generation: Option<GenerationRecord>,
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
            .field("state_epoch", &self.state_epoch)
            .field("world", &"<redacted>")
            .field("reports", &"<redacted>")
            .field("generation", &self.generation)
            .finish()
    }
}
