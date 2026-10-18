use serde::Serialize;

use super::super::{Cell, Direction, EntityId, Position, Seat, Tick};
use super::{EffectViews, InventoryView, PickupKind};

pub const ARENA_TERRAIN_CELLS: usize = 195;
pub const MAX_BLOCKS: usize = 195;
pub const MAX_BOMBS: usize = 16;
pub const MAX_FLAMES: usize = 195;
pub const MAX_PICKUPS: usize = 195;
pub const MAX_RUNNING_UPDATES: u16 = 10_800;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DamageGraceTicks(u8);

impl DamageGraceTicks {
    pub const fn new(value: u8) -> Result<Self, super::super::InvalidScalar> {
        if value <= 60 {
            Ok(Self(value))
        } else {
            Err(super::super::InvalidScalar)
        }
    }
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GameState {
    pub running_updates: u16,
    pub remaining_ticks: u16,
    pub physical_tick: Tick,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ActorView {
    Visible {
        id: EntityId,
        seat: Seat,
        position: Position,
        heading: Direction,
        #[serde(rename = "speedLevel")]
        speed_level: u8,
        alive: bool,
        #[serde(rename = "bombCapacity")]
        bomb_capacity: u8,
        #[serde(rename = "bombRange")]
        bomb_range: u8,
        #[serde(rename = "liveBombCount")]
        live_bomb_count: u8,
        inventory: InventoryView,
        effects: EffectViews,
        #[serde(rename = "damageGraceRemainingTicks")]
        damage_grace_remaining_ticks: DamageGraceTicks,
    },
    Hidden {
        id: EntityId,
        seat: Seat,
        alive: bool,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisibleActorView {
    pub id: EntityId,
    pub seat: Seat,
    pub position: Position,
    pub heading: Direction,
    pub speed_level: u8,
    pub alive: bool,
    pub bomb_capacity: u8,
    pub bomb_range: u8,
    pub live_bomb_count: u8,
    pub inventory: InventoryView,
    pub effects: EffectViews,
    pub damage_grace_remaining_ticks: DamageGraceTicks,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HiddenActorView {
    pub id: EntityId,
    pub seat: Seat,
    pub alive: bool,
}

impl From<VisibleActorView> for ActorView {
    fn from(actor: VisibleActorView) -> Self {
        Self::Visible {
            id: actor.id,
            seat: actor.seat,
            position: actor.position,
            heading: actor.heading,
            speed_level: actor.speed_level,
            alive: actor.alive,
            bomb_capacity: actor.bomb_capacity,
            bomb_range: actor.bomb_range,
            live_bomb_count: actor.live_bomb_count,
            inventory: actor.inventory,
            effects: actor.effects,
            damage_grace_remaining_ticks: actor.damage_grace_remaining_ticks,
        }
    }
}
impl From<HiddenActorView> for ActorView {
    fn from(actor: HiddenActorView) -> Self {
        Self::Hidden {
            id: actor.id,
            seat: actor.seat,
            alive: actor.alive,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct BombTrigger(BombTriggerWire);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
enum BombTriggerWire {
    Timed {
        #[serde(rename = "remainingTicks")]
        remaining_ticks: u16,
        paused: bool,
    },
    Remote,
    Contact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BombTriggerObservation {
    Timed { remaining_ticks: u16, paused: bool },
    Remote,
    Contact,
}

impl BombTrigger {
    pub const fn timed(
        remaining_ticks: u16,
        paused: bool,
    ) -> Result<Self, super::super::InvalidScalar> {
        if remaining_ticks <= 600 {
            Ok(Self(BombTriggerWire::Timed {
                remaining_ticks,
                paused,
            }))
        } else {
            Err(super::super::InvalidScalar)
        }
    }
    #[must_use]
    pub const fn remote() -> Self {
        Self(BombTriggerWire::Remote)
    }
    #[must_use]
    pub const fn contact() -> Self {
        Self(BombTriggerWire::Contact)
    }

    pub(crate) const fn observation(self) -> BombTriggerObservation {
        match self.0 {
            BombTriggerWire::Timed {
                remaining_ticks,
                paused,
            } => BombTriggerObservation::Timed {
                remaining_ticks,
                paused,
            },
            BombTriggerWire::Remote => BombTriggerObservation::Remote,
            BombTriggerWire::Contact => BombTriggerObservation::Contact,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum BlastKind {
    Standard,
    Piercing,
    Power,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BombView {
    pub id: EntityId,
    pub cell: Cell,
    pub owner: EntityId,
    pub range: u8,
    pub placed_at: Tick,
    pub trigger: BombTrigger,
    pub blast: BlastKind,
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
