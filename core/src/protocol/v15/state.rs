use serde::Serialize;

use super::{ArenaView, BombView, FlameView, OilBarrelView, PickupView};
use crate::protocol::{Direction, EntityId, Position, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StormCloudView {
    pub position: Position,
    pub direction: Direction,
    pub leg_remaining_ticks: u16,
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
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum ActorView {
    Visible {
        id: EntityId,
        seat: super::Seat,
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
        inventory: super::InventoryView,
        effects: super::EffectViews,
        #[serde(rename = "damageGraceRemainingTicks")]
        damage_grace_remaining_ticks: super::DamageGraceTicks,
        #[serde(rename = "voluntaryActionsFrozen")]
        voluntary_actions_frozen: bool,
        #[serde(rename = "lastTeleportTick")]
        last_teleport_tick: Option<Tick>,
        #[serde(rename = "stormSlowRemainingTicks")]
        storm_slow_remaining_ticks: u16,
    },
    Hidden {
        id: EntityId,
        seat: super::Seat,
        alive: bool,
    },
}
