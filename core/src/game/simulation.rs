use std::collections::BTreeSet;

#[cfg(test)]
use super::arena::Terrain;
#[cfg(test)]
use super::bombs::Bomb;
use super::events::StepEvent;
use super::geometry::MovementTrace;
use super::input::{Direction, HeldDirections};
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
    BombIdMismatch { key: EntityId, id: EntityId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepError {
    TooManyInputs { count: usize, maximum: usize },
    TickOverflow,
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
}

impl State {
    pub fn new(
        rules: RulesConfig,
        world: World,
        actors: [ActorInit; 2],
    ) -> Result<Self, StateError> {
        if actors[0].id == actors[1].id {
            return Err(StateError::DuplicateActorId(actors[0].id));
        }
        if actors[0].seat == actors[1].seat {
            return Err(StateError::DuplicateSeat(actors[0].seat));
        }
        for (&key, bomb) in &world.bombs {
            if key != bomb.id() {
                return Err(StateError::BombIdMismatch { key, id: bomb.id() });
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
            tick: Tick(0),
            next_entity_id,
            running_updates: 0,
            result: None,
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
    pub fn live_bomb_count(&self, seat: SeatId) -> usize {
        let owner = self.actors[seat.index()].id;
        self.world
            .bombs
            .values()
            .filter(|bomb| bomb.owner() == owner)
            .count()
    }

    #[cfg(test)]
    pub(crate) fn insert_test_bomb(
        &mut self,
        id: EntityId,
        cell: CellId,
        owner: EntityId,
        range: u16,
        placed_at: Tick,
        detonates_at: Tick,
    ) -> Result<(), StateError> {
        if self.actors.iter().any(|actor| actor.id == id) || self.world.bombs.contains_key(&id) {
            return Err(StateError::DuplicateEntityId(id));
        }
        if !self.actors.iter().any(|actor| actor.id == owner) {
            return Err(StateError::UnknownBombOwner(owner));
        }
        if self.world.arena.terrain(cell) != Some(Terrain::Floor)
            || self.world.arena.blocks().contains(&cell)
        {
            return Err(StateError::IllegalBombCell(cell));
        }
        if self.world.bombs.values().any(|bomb| bomb.cell == cell) {
            return Err(StateError::OccupiedBombCell(cell));
        }

        self.insert_bomb(Bomb {
            id,
            cell,
            owner,
            range,
            placed_at,
            detonates_at,
        });
        Ok(())
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
        }
    }
}

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
#[path = "simulation_tests.rs"]
mod tests;
