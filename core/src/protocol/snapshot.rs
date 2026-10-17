use serde::Serialize;

use super::scalars::{Cell, Direction, EntityId, Position, Seat, Tick};

pub const ARENA_TERRAIN_CELLS: usize = 195;
pub const MAX_BLOCKS: usize = 195;
pub const MAX_BOMBS: usize = 16;
pub const MAX_FLAMES: usize = 195;
pub const MAX_PICKUPS: usize = 195;
pub const MAX_RUNNING_UPDATES: u16 = 10_800;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameState {
    pub running_updates: u16,
    pub remaining_ticks: u16,
    pub arena: ArenaView,
    pub actors: [ActorView; 2],
    pub bombs: Vec<BombView>,
    pub flames: Vec<FlameView>,
    pub pickups: Vec<PickupView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArenaView {
    pub width: u8,
    pub height: u8,
    pub terrain: Vec<Terrain>,
    pub blocks: Vec<Cell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActorView {
    pub id: EntityId,
    pub seat: Seat,
    pub position: Position,
    pub heading: Direction,
    pub speed_level: u8,
    pub alive: bool,
    pub bomb_capacity: u8,
    pub bomb_range: u8,
    pub live_bomb_count: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BombView {
    pub id: EntityId,
    pub cell: Cell,
    pub owner: EntityId,
    pub range: u8,
    pub placed_at: Tick,
    pub detonates_at: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlameView {
    pub cell: Cell,
    pub expires_at: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Terrain {
    Floor,
    Wall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PickupView {
    pub cell: Cell,
    pub kind: PickupKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PickupKind {
    BombCapacity,
    BlastRange,
    MovementSpeed,
}
