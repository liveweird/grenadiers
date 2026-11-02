use std::collections::BTreeSet;

use super::assignment::RoundAssignment;
use super::damage::DamageGrace;
use super::effects::{EffectiveSpeed, TimedEffects};
use super::geometry::MovementTrace;
use super::input::{Direction, HeldDirections};
use super::inventory::Inventory;
use super::storm_cloud::StormCloudState;
use super::types::{CellId, EntityId, PlayerCount, Position, SeatId, SpeedLevel, Tick};
use super::world::World;
use super::{lairs::LairState, movement, rules::RulesConfig};
use simulation_spike_pits::SpikePitState;

#[path = "simulation_contract.rs"]
mod contract;
pub use contract::{
    ActorInit, InputAction, MAX_INPUTS_PER_TICK, SeatInput, StateError, StepError, StepReport,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TurnAssist {
    pub(crate) direction: Direction,
    pub(crate) target: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Actor {
    pub(crate) id: EntityId,
    pub(crate) seat: SeatId,
    pub(crate) position: Position,
    pub(crate) heading: Direction,
    pub(crate) speed: SpeedLevel,
    pub(crate) held: HeldDirections,
    pub(crate) residual: u16,
    pub(crate) pending_turn: Option<TurnAssist>,
    pub(crate) bomb_grace: BTreeSet<EntityId>,
    pub(crate) soft_wall_grace: BTreeSet<CellId>,
    pub(crate) unlit_oil_grace: BTreeSet<CellId>,
    capacity: u16,
    range: u16,
    alive: bool,
    inventory: Inventory,
    effects: TimedEffects,
    damage_grace: Option<DamageGrace>,
    last_successful_punch_tick: Option<Tick>,
    contact_punch_latches: BTreeSet<EntityId>,
    last_successful_shove_tick: Option<Tick>,
    last_teleport_tick: Option<Tick>,
    storm_slow_until: Option<Tick>,
}

impl Actor {
    #[must_use]
    pub const fn id(&self) -> EntityId {
        self.id
    }
    #[must_use]
    pub const fn seat(&self) -> SeatId {
        self.seat
    }
    #[must_use]
    pub const fn position(&self) -> Position {
        self.position
    }
    #[must_use]
    pub const fn heading(&self) -> Direction {
        self.heading
    }
    #[must_use]
    pub const fn speed(&self) -> SpeedLevel {
        self.speed
    }
    #[must_use]
    pub const fn bomb_capacity(&self) -> u16 {
        self.capacity
    }
    #[must_use]
    pub const fn bomb_range(&self) -> u16 {
        self.range
    }
    #[must_use]
    pub const fn alive(&self) -> bool {
        self.alive
    }

    #[must_use]
    pub const fn inventory(&self) -> &Inventory {
        &self.inventory
    }

    #[must_use]
    pub const fn effects(&self) -> &TimedEffects {
        &self.effects
    }

    #[must_use]
    pub fn effective_speed(&self, at: Tick) -> EffectiveSpeed {
        EffectiveSpeed::from_modifiers(
            self.speed,
            self.effects.slow_foot_stacks(at),
            self.storm_slow_active(at),
        )
    }

    #[must_use]
    pub const fn storm_slow_until(&self) -> Option<Tick> {
        self.storm_slow_until
    }

    #[must_use]
    pub fn storm_slow_active(&self, at: Tick) -> bool {
        self.storm_slow_until.is_some_and(|expiry| at < expiry)
    }

    #[must_use]
    pub fn effective_bomb_range(&self, at: Tick) -> u16 {
        self.effects.bomb_range(self.range, at)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawReason {
    MutualElimination,
    Timeout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoundResult {
    Winner(SeatId),
    Draw(DrawReason),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    rules: RulesConfig,
    world: World,
    actors: Vec<Actor>,
    player_count: PlayerCount,
    tick: Tick,
    next_entity_id: u64,
    running_updates: u64,
    result: Option<RoundResult>,
    assignment: RoundAssignment,
    last_teleport_tick: Option<Tick>,
    construction_tick: Tick,
    randomness: Option<Box<random::SimulationRandomness>>,
    storm_cloud: Option<Box<StormCloudState>>,
    lairs: Option<LairState>,
    spike_pits: Option<SpikePitState>,
}

impl State {
    pub fn new(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<ActorInit>>,
    ) -> Result<Self, StateError> {
        Self::new_at_tick(rules, world, actors, Tick(0))
    }
    pub fn new_at_tick(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<ActorInit>>,
        tick: Tick,
    ) -> Result<Self, StateError> {
        Self::new_at_tick_with_assignment(rules, world, actors, tick, RoundAssignment::first())
    }
    pub fn new_at_tick_with_assignment(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<ActorInit>>,
        tick: Tick,
        assignment: RoundAssignment,
    ) -> Result<Self, StateError> {
        Self::new_at_tick_with_assignment_and_randomness(
            rules, world, actors, tick, assignment, None,
        )
    }

    pub fn new_at_tick_with_assignment_and_seed(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<ActorInit>>,
        tick: Tick,
        assignment: RoundAssignment,
        seed: SimulationSeed,
    ) -> Result<Self, StateError> {
        Self::new_at_tick_with_assignment_and_randomness(
            rules,
            world,
            actors,
            tick,
            assignment,
            Some(Box::new(random::SimulationRandomness::new(seed))),
        )
    }
}

#[path = "simulation_actions.rs"]
mod actions;
#[path = "simulation_actor.rs"]
mod actor_state;
#[path = "simulation_lairs.rs"]
mod lair_step;
#[path = "simulation_manual_actions.rs"]
mod manual_actions;
#[cfg(test)]
include!("simulation_test_modules.rs");
mod contacts;
#[path = "simulation_explosions.rs"]
mod explosion_step;
#[path = "simulation_observations.rs"]
mod observations;
#[path = "simulation_oil.rs"]
mod oil_step;
#[path = "simulation_participation.rs"]
mod participation;
#[path = "simulation_pickups.rs"]
mod pickup_step;
#[path = "simulation_punch.rs"]
mod punch;
#[path = "simulation_random.rs"]
mod random;
#[path = "simulation_shove.rs"]
mod shove;
#[path = "shove_geometry.rs"]
mod shove_geometry;
#[path = "simulation_spike_pits.rs"]
mod simulation_spike_pits;
#[path = "simulation_storm.rs"]
mod storm_step;

pub use super::storm_cloud::{
    STORM_CLOUD_HALF_EXTENT, STORM_CLOUD_LEG_TICKS, STORM_CLOUD_RANDOMNESS_VERSION,
    STORM_CLOUD_SPEED_PER_TICK, STORM_SLOW_TICKS, StormCloudView,
};
pub use participation::RoundActorInit;
pub use random::{
    SIMULATION_RANDOMNESS_VERSION, SIMULATION_SEED_DERIVATION_VERSION, SimulationSeed,
};
pub(crate) use shove_geometry::{stationary_contact_entry, stationary_sweep_contact};
#[path = "simulation_kick_requests.rs"]
mod kick_requests;
#[path = "simulation_step.rs"]
mod step;
#[path = "simulation_stopwatch.rs"]
mod stopwatch;
#[path = "simulation_teleport.rs"]
mod teleport;
#[path = "simulation_validation.rs"]
mod validation;

#[path = "simulation_construction.rs"]
mod construction;
#[path = "simulation_movement.rs"]
mod movement_step;
#[path = "simulation_player_movement.rs"]
mod player_movement;
