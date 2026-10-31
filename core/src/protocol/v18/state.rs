use serde::Serialize;

use super::{ArenaView, BombView, FlameView, OilBarrelView, PickupView};
use crate::protocol::{Cell, Direction, EntityId, Position, Tick};

pub use crate::protocol::v15::{ActorView, StormCloudView};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LairView {
    pub id: u8,
    pub cell: Cell,
    pub spawn_remaining_ticks: Option<u16>,
    pub living_mobs: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MobView {
    pub id: EntityId,
    pub lair_id: u8,
    pub position: Position,
    pub heading: Direction,
    pub frozen: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SpikePitPhase {
    Idle,
    Warning,
    Raised,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpikePitView {
    pub cell: Cell,
    pub phase: SpikePitPhase,
    pub remaining_ticks: u16,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameState {
    pub running_updates: u16,
    pub remaining_ticks: u16,
    pub physical_tick: Tick,
    pub arena: ArenaView,
    pub actors: Vec<ActorView>,
    pub bombs: Vec<BombView>,
    pub flames: Vec<FlameView>,
    pub oil_barrels: Vec<OilBarrelView>,
    pub pickups: Vec<PickupView>,
    pub storm_cloud: Option<StormCloudView>,
    pub lairs: Vec<LairView>,
    pub mobs: Vec<MobView>,
    pub spike_pits: Vec<SpikePitView>,
}
