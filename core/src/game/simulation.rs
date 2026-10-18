use std::collections::BTreeSet;

use super::assignment::RoundAssignment;
use super::bombs::BombTrigger;
use super::damage::DamageGrace;
use super::effects::{EffectiveSpeed, TimedEffects};
use super::events::StepEvent;
use super::geometry::MovementTrace;
use super::input::{Direction, HeldDirections};
use super::inventory::Inventory;
use super::movement;
use super::rules::RulesConfig;
use super::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use super::world::World;

pub const MAX_INPUTS_PER_TICK: usize = 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorInit {
    pub id: EntityId,
    pub seat: SeatId,
    pub position: Position,
    pub heading: Direction,
    pub speed: SpeedLevel,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputAction {
    Press(Direction),
    Release(Direction),
    Clear,
    PlaceBomb,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeatInput {
    pub seat: SeatId,
    pub action: InputAction,
}

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
    capacity: u16,
    range: u16,
    alive: bool,
    inventory: Inventory,
    effects: TimedEffects,
    damage_grace: Option<DamageGrace>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    DuplicateActorId(EntityId),
    DuplicateSeat(SeatId),
    IllegalActorPosition(EntityId),
    DuplicateEntityId(EntityId),
    IllegalBombCell(CellId),
    OccupiedBombCell(CellId),
    UnknownBombOwner(EntityId),
    BombIdMismatch {
        key: EntityId,
        id: EntityId,
    },
    UnsupportedBombTrigger(EntityId),
    UnsupportedEnabledItems(crate::game::EnabledItems),
    UnsupportedPickup {
        cell: CellId,
        kind: crate::game::PickupKind,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    TooManyInputs { count: usize, maximum: usize },
    TickOverflow,
    UnsupportedBombTrigger(EntityId),
    EffectExpiryOverflow(super::effects::EffectKind),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepReport {
    pub tick: Tick,
    pub traces: [MovementTrace; 2],
    pub events: Vec<StepEvent>,
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
}

impl State {
    pub fn new(
        rules: RulesConfig,
        world: World,
        actors: [ActorInit; 2],
    ) -> Result<Self, StateError> {
        Self::new_at_tick(rules, world, actors, Tick(0))
    }

    /// Creates a round at an existing absolute room tick.
    ///
    /// The round-local running update count always starts at zero.
    pub fn new_at_tick(
        rules: RulesConfig,
        world: World,
        actors: [ActorInit; 2],
        tick: Tick,
    ) -> Result<Self, StateError> {
        Self::new_at_tick_with_assignment(rules, world, actors, tick, RoundAssignment::first())
    }

    /// Creates a round at an absolute room tick with explicit parity-based roles.
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
            if !matches!(bomb.trigger(), BombTrigger::Timed { .. }) {
                return Err(StateError::UnsupportedBombTrigger(bomb.id()));
            }
        }
        for actor in actors {
            if world.bombs.contains_key(&actor.id) {
                return Err(StateError::DuplicateEntityId(actor.id));
            }
            if !movement::position_is_free(&world, actor.position, &BTreeSet::new()) {
                return Err(StateError::IllegalActorPosition(actor.id));
            }
        }
        for bomb in world.bombs.values() {
            if !actors.iter().any(|actor| actor.id == bomb.owner()) {
                return Err(StateError::UnknownBombOwner(bomb.owner()));
            }
        }

        let mut canonical = actors;
        canonical.sort_by_key(|actor| actor.seat.index());
        let next_entity_id = world
            .bombs
            .keys()
            .next_back()
            .map_or(0, |id| u64::from(id.0) + 1);
        Ok(Self {
            rules,
            world,
            actors: canonical.map(|actor| Actor::new(actor, rules)),
            tick,
            next_entity_id,
            running_updates: 0,
            result: None,
            assignment,
        })
    }

    #[must_use]
    pub const fn rules(&self) -> &RulesConfig {
        &self.rules
    }

    #[must_use]
    pub const fn world(&self) -> &World {
        &self.world
    }

    #[must_use]
    pub const fn actors(&self) -> &[Actor; 2] {
        &self.actors
    }

    #[must_use]
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    #[must_use]
    pub const fn running_updates(&self) -> u64 {
        self.running_updates
    }

    #[must_use]
    pub const fn result(&self) -> Option<RoundResult> {
        self.result
    }

    #[must_use]
    pub const fn round_assignment(&self) -> RoundAssignment {
        self.assignment
    }

    #[must_use]
    pub fn live_bomb_count(&self, seat: SeatId) -> usize {
        let owner = self.actors[seat.index()].id;
        self.world
            .bombs
            .values()
            .filter(|bomb| bomb.owner() == owner)
            .count()
    }
}

impl Actor {
    fn new(init: ActorInit, rules: RulesConfig) -> Self {
        Self {
            id: init.id,
            seat: init.seat,
            position: init.position,
            heading: init.heading,
            speed: init.speed,
            held: HeldDirections::default(),
            residual: 0,
            pending_turn: None,
            bomb_grace: BTreeSet::new(),
            capacity: rules.bomb_capacity(),
            range: rules.bomb_range(),
            alive: true,
            inventory: Inventory::default(),
            effects: TimedEffects::default(),
            damage_grace: None,
        }
    }
}

#[path = "simulation_actions.rs"]
mod actions;
#[cfg(test)]
#[path = "bomb_order_acceptance_tests.rs"]
mod bomb_order_acceptance_tests;
#[cfg(test)]
#[path = "bomb_trigger_tests.rs"]
mod bomb_trigger_tests;
#[cfg(test)]
#[path = "simulation_constructor_tests.rs"]
mod constructor_tests;
#[cfg(test)]
#[path = "damage_tests.rs"]
mod damage_tests;
#[path = "simulation_explosions.rs"]
mod explosion_step;
#[cfg(test)]
#[path = "flame_tests.rs"]
mod flame_tests;
#[cfg(test)]
#[path = "item_foundation_tests.rs"]
mod item_foundation_tests;
#[path = "simulation_observations.rs"]
mod observations;
#[path = "simulation_pickups.rs"]
mod pickup_step;
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
#[path = "round_result_tests.rs"]
mod round_result_tests;
#[path = "simulation_step.rs"]
mod step;
#[cfg(test)]
#[path = "step_event_tests.rs"]
mod step_event_tests;
#[cfg(test)]
#[path = "simulation_test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "upgrade_tests.rs"]
mod upgrade_tests;
