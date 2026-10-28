use std::collections::BTreeSet;

use super::assignment::RoundAssignment;
use super::damage::DamageGrace;
use super::effects::{EffectiveSpeed, TimedEffects};
use super::geometry::MovementTrace;
use super::input::{Direction, HeldDirections};
use super::inventory::Inventory;
use super::movement;
use super::rules::RulesConfig;
use super::types::{CellId, EntityId, PlayerCount, Position, SeatId, SpeedLevel, Tick};
use super::world::World;

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
    last_successful_shove_tick: Option<Tick>,
    last_teleport_tick: Option<Tick>,
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
        self.effects.effective_speed(self.speed, at)
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

    fn new_at_tick_with_assignment_and_randomness(
        rules: RulesConfig,
        world: World,
        actors: impl Into<Vec<ActorInit>>,
        tick: Tick,
        assignment: RoundAssignment,
        randomness: Option<Box<random::SimulationRandomness>>,
    ) -> Result<Self, StateError> {
        let mut actors = actors.into();
        let player_count = PlayerCount::try_from(actors.len())
            .map_err(|()| StateError::InvalidPlayerCount(actors.len()))?;
        if world.arena.spawns().len() != actors.len() {
            return Err(StateError::SpawnCountMismatch {
                actors: actors.len(),
                spawns: world.arena.spawns().len(),
            });
        }
        if rules.version() < 6 && player_count != PlayerCount::Two {
            return Err(StateError::UnsupportedPlayerCount {
                version: rules.version(),
                count: player_count,
            });
        }
        let enabled = rules.enabled_items();
        if rules.version() >= 6 && enabled.contains_teleporter() && randomness.is_none() {
            return Err(StateError::MissingSimulationRandomness);
        }
        let mut ids = BTreeSet::new();
        for actor in &actors {
            if !ids.insert(actor.id) {
                return Err(StateError::DuplicateActorId(actor.id));
            }
        }
        if let Some(enabled) = validation::unsupported_enabled_items(rules) {
            return Err(StateError::UnsupportedEnabledItems(enabled));
        }
        for (cell, pickup) in world.pickups.iter() {
            if !pickup.enabled_by(rules.enabled_items()) {
                return Err(StateError::UnsupportedPickup { cell, kind: pickup });
            }
        }
        if world.oil.iter().next().is_some() && !rules.enabled_items().contains_oil_barrel() {
            return Err(StateError::UnsupportedOilBarrels);
        }
        if let Some(cell) = validation::invalid_oil(&world, tick) {
            return Err(StateError::InvalidOilBarrel(cell));
        }
        let mut seats = BTreeSet::new();
        for actor in &actors {
            if !seats.insert(actor.seat) {
                return Err(StateError::DuplicateSeat(actor.seat));
            }
        }
        actors.sort_by_key(|actor| actor.seat.index());
        if actors
            .iter()
            .map(|actor| actor.seat)
            .ne(player_count.seats().iter().copied())
        {
            return Err(StateError::NonCanonicalSeats);
        }
        for (&key, bomb) in &world.bombs {
            if key != bomb.id() {
                return Err(StateError::BombIdMismatch { key, id: bomb.id() });
            }
        }
        if let Some(id) = validation::unsupported_trigger(rules, &world) {
            return Err(StateError::UnsupportedBombTrigger(id));
        }
        if let Some(id) = validation::unsupported_blast(rules, &world) {
            return Err(StateError::UnsupportedBombBlast(id));
        }
        if let Some(owner) = validation::duplicate_remote_owner(&world) {
            return Err(StateError::DuplicateRemoteOwner(owner));
        }
        for actor in &actors {
            if world.bombs.contains_key(&actor.id) {
                return Err(StateError::DuplicateEntityId(actor.id));
            }
            if !movement::position_is_free(
                &world,
                actor.position,
                &BTreeSet::new(),
                false,
                &BTreeSet::new(),
                &BTreeSet::new(),
            ) {
                return Err(StateError::IllegalActorPosition(actor.id));
            }
        }
        for bomb in world.bombs.values() {
            if !actors.iter().any(|actor| actor.id == bomb.owner()) {
                return Err(StateError::UnknownBombOwner(bomb.owner()));
            }
        }
        if let Some(id) = validation::unsupported_kick_history(rules, &world) {
            return Err(StateError::UnsupportedBombKickHistory(id));
        }
        if let Some(id) = validation::unsupported_punch_history(rules, &world) {
            return Err(StateError::UnsupportedBombPunchHistory(id));
        }

        let next_entity_id = world
            .bombs
            .keys()
            .next_back()
            .map_or(0, |id| u64::from(id.0) + 1);
        let mut initialized: Vec<_> = actors
            .into_iter()
            .map(|actor| Actor::new(actor, rules))
            .collect();
        contacts::initialize_grace(&world, &mut initialized);
        Ok(Self {
            rules,
            world,
            actors: initialized,
            player_count,
            tick,
            next_entity_id,
            running_updates: 0,
            result: None,
            assignment,
            last_teleport_tick: None,
            construction_tick: tick,
            randomness: (rules.version() >= 6).then_some(randomness).flatten(),
        })
    }
}

#[path = "simulation_actions.rs"]
mod actions;
#[path = "simulation_actor.rs"]
mod actor_state;
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

pub use participation::RoundActorInit;
pub use random::{
    SIMULATION_RANDOMNESS_VERSION, SIMULATION_SEED_DERIVATION_VERSION, SimulationSeed,
};
pub(crate) use shove_geometry::stationary_contact_entry;
#[path = "simulation_step.rs"]
mod step;
#[path = "simulation_stopwatch.rs"]
mod stopwatch;
#[path = "simulation_teleport.rs"]
mod teleport;
#[path = "simulation_validation.rs"]
mod validation;
