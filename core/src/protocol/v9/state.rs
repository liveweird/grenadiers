use serde::Serialize;

use super::super::{Direction, EntityId, Position, Seat, Tick};
use super::{
    ArenaView, BombView, DamageGraceTicks, EffectViews, FlameView, HiddenActorView, InventoryView,
    PickupView,
};

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
        #[serde(rename = "voluntaryActionsFrozen")]
        voluntary_actions_frozen: bool,
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
    pub voluntary_actions_frozen: bool,
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
            voluntary_actions_frozen: actor.voluntary_actions_frozen,
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
