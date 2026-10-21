use std::collections::BTreeSet;

use super::assignment::RoundAssignment;
use super::damage::DamageGrace;
use super::effects::{EffectiveSpeed, TimedEffects};
use super::input::{Direction, HeldDirections};
use super::inventory::Inventory;
use super::movement;
use super::rules::RulesConfig;
use super::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
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
    capacity: u16,
    range: u16,
    alive: bool,
    inventory: Inventory,
    effects: TimedEffects,
    damage_grace: Option<DamageGrace>,
    last_successful_punch_tick: Option<Tick>,
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
    actors: [Actor; 2],
    tick: Tick,
    next_entity_id: u64,
    running_updates: u64,
    result: Option<RoundResult>,
    assignment: RoundAssignment,
    last_teleport_tick: Option<Tick>,
}

impl State {
    pub fn new(
        rules: RulesConfig,
        world: World,
        actors: [ActorInit; 2],
    ) -> Result<Self, StateError> {
        Self::new_at_tick(rules, world, actors, Tick(0))
    }
    pub fn new_at_tick(
        rules: RulesConfig,
        world: World,
        actors: [ActorInit; 2],
        tick: Tick,
    ) -> Result<Self, StateError> {
        Self::new_at_tick_with_assignment(rules, world, actors, tick, RoundAssignment::first())
    }
    pub fn new_at_tick_with_assignment(
        rules: RulesConfig,
        world: World,
        actors: [ActorInit; 2],
        tick: Tick,
        assignment: RoundAssignment,
    ) -> Result<Self, StateError> {
        if actors[0].id == actors[1].id {
            return Err(StateError::DuplicateActorId(actors[0].id));
        }
        if rules.version() < 5
            && rules.room_settings().is_some_and(|settings| {
                settings.enabled_items() != crate::game::EnabledItems::EMPTY
            })
        {
            return Err(StateError::UnsupportedEnabledItems(
                rules
                    .room_settings()
                    .expect("checked above")
                    .enabled_items(),
            ));
        }
        for (cell, pickup) in world.pickups.iter() {
            if !pickup.enabled_by(rules.enabled_items()) {
                return Err(StateError::UnsupportedPickup { cell, kind: pickup });
            }
        }
        if actors[0].seat == actors[1].seat {
            return Err(StateError::DuplicateSeat(actors[0].seat));
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
        for actor in actors {
            if world.bombs.contains_key(&actor.id) {
                return Err(StateError::DuplicateEntityId(actor.id));
            }
            if !movement::position_is_free(
                &world,
                actor.position,
                &BTreeSet::new(),
                false,
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

        let mut canonical = actors;
        canonical.sort_by_key(|actor| actor.seat.index());
        let next_entity_id = world
            .bombs
            .keys()
            .next_back()
            .map_or(0, |id| u64::from(id.0) + 1);
        let mut initialized = canonical.map(|actor| Actor::new(actor, rules));
        contacts::initialize_grace(&world, &mut initialized);
        Ok(Self {
            rules,
            world,
            actors: initialized,
            tick,
            next_entity_id,
            running_updates: 0,
            result: None,
            assignment,
            last_teleport_tick: None,
        })
    }
}

#[path = "simulation_actions.rs"]
mod actions;
#[path = "simulation_actor.rs"]
mod actor_state;
#[cfg(test)]
#[path = "bomb_order_acceptance_tests.rs"]
mod bomb_order_acceptance_tests;
#[cfg(test)]
#[path = "bomb_trigger_tests.rs"]
mod bomb_trigger_tests;
#[cfg(test)]
#[path = "simulation_constructor_tests.rs"]
mod constructor_tests;
mod contacts;
#[cfg(test)]
#[path = "damage_tests.rs"]
mod damage_tests;
#[path = "simulation_explosions.rs"]
mod explosion_step;
#[cfg(test)]
#[path = "flame_tests.rs"]
mod flame_tests;
#[cfg(test)]
#[path = "golden_flame_tests.rs"]
mod golden_flame_tests;
#[cfg(test)]
#[path = "simulation/invisibility_cloak_acceptance_tests.rs"]
mod invisibility_cloak_acceptance_tests;
#[cfg(test)]
#[path = "item_foundation_tests.rs"]
mod item_foundation_tests;
#[cfg(test)]
#[path = "kick_validation_tests.rs"]
mod kick_validation_tests;
#[cfg(test)]
#[path = "simulation/landmine_acceptance_tests.rs"]
mod landmine_acceptance_tests;
#[cfg(test)]
#[path = "simulation/landmine_route_tests.rs"]
mod landmine_route_tests;
#[cfg(test)]
mod landmine_tests;
#[path = "simulation_observations.rs"]
mod observations;
#[path = "simulation_pickups.rs"]
mod pickup_step;
#[path = "simulation_punch.rs"]
mod punch;
#[cfg(test)]
#[path = "pickup_tests.rs"]
mod pickup_tests;
#[cfg(test)]
#[path = "placement_tests.rs"]
mod placement_tests;
#[cfg(test)]
#[path = "projection_tests.rs"]
mod projection_tests;
#[cfg(test)]
#[path = "punch_tests.rs"]
mod punch_tests;
#[cfg(test)]
#[path = "punch_rollback_tests.rs"]
mod punch_rollback_tests;
#[cfg(test)]
#[path = "punch_validation_tests.rs"]
mod punch_validation_tests;
#[cfg(test)]
#[path = "punch_wave_tests.rs"]
mod punch_wave_tests;
#[cfg(test)]
#[path = "round_result_tests.rs"]
mod round_result_tests;
#[cfg(test)]
#[path = "short_fuse_tests.rs"]
mod short_fuse_tests;
#[path = "simulation_step.rs"]
mod step;
#[cfg(test)]
#[path = "step_event_tests.rs"]
mod step_event_tests;
#[path = "simulation_stopwatch.rs"]
mod stopwatch;
#[cfg(test)]
#[path = "stopwatch_interaction_tests.rs"]
mod stopwatch_interaction_tests;
#[cfg(test)]
#[path = "simulation/stopwatch_route_tests.rs"]
mod stopwatch_route_tests;
#[cfg(test)]
#[path = "stopwatch_tests.rs"]
mod stopwatch_tests;
#[path = "simulation_teleport.rs"]
mod teleport;
#[cfg(test)]
#[path = "simulation/teleporter_acceptance_tests.rs"]
mod teleporter_acceptance_tests;
#[cfg(test)]
#[path = "simulation/teleporter_route_tests.rs"]
mod teleporter_route_tests;
#[cfg(test)]
#[path = "teleporter_tests.rs"]
mod teleporter_tests;
#[cfg(test)]
#[path = "simulation_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "upgrade_tests.rs"]
mod upgrade_tests;
#[path = "simulation_validation.rs"]
mod validation;
#[cfg(test)]
#[path = "simulation/wall_pass_contract_tests.rs"]
mod wall_pass_contract_tests;
#[cfg(test)]
#[path = "simulation/wall_pass_route_tests.rs"]
mod wall_pass_route_tests;
#[cfg(test)]
#[path = "wall_pass_tests.rs"]
mod wall_pass_tests;
