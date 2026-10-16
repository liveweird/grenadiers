use std::collections::BTreeSet;

use super::arena::Terrain;
use super::geometry::MovementTrace;
use super::input::{Direction, HeldDirections};
use super::movement;
use super::rules::RulesConfig;
use super::types::{CellId, EntityId, Position, SeatId, SpeedLevel, Tick};
use super::world::{BombObstacle, World};

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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateError {
    DuplicateActorId(EntityId),
    DuplicateSeat(SeatId),
    IllegalActorPosition(EntityId),
    DuplicateEntityId(EntityId),
    IllegalBombCell(CellId),
    OccupiedBombCell(CellId),
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    rules: RulesConfig,
    world: World,
    actors: [Actor; 2],
    tick: Tick,
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
        for actor in actors {
            if world.bombs.contains_key(&actor.id) {
                return Err(StateError::DuplicateEntityId(actor.id));
            }
            if !movement::position_is_free(&world, actor.position, &BTreeSet::new()) {
                return Err(StateError::IllegalActorPosition(actor.id));
            }
        }

        let mut canonical = actors;
        canonical.sort_by_key(|actor| actor.seat.index());
        Ok(Self {
            rules,
            world,
            actors: canonical.map(Actor::from),
            tick: Tick(0),
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

    pub fn add_bomb_obstacle(&mut self, id: EntityId, cell: CellId) -> Result<(), StateError> {
        if self.actors.iter().any(|actor| actor.id == id) || self.world.bombs.contains_key(&id) {
            return Err(StateError::DuplicateEntityId(id));
        }
        if self.world.arena.terrain(cell) != Some(Terrain::Floor)
            || self.world.arena.blocks().contains(&cell)
        {
            return Err(StateError::IllegalBombCell(cell));
        }
        if self.world.bombs.values().any(|bomb| bomb.cell == cell) {
            return Err(StateError::OccupiedBombCell(cell));
        }

        let obstacle = BombObstacle { id, cell };
        self.world.bombs.insert(id, obstacle);
        for actor in &mut self.actors {
            if movement::position_overlaps_cell(actor.position, cell) {
                actor.bomb_grace.insert(id);
            }
        }
        Ok(())
    }

    pub fn step(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if inputs.len() > MAX_INPUTS_PER_TICK {
            return Err(StepError::TooManyInputs {
                count: inputs.len(),
                maximum: MAX_INPUTS_PER_TICK,
            });
        }
        let next_tick = Tick(self.tick.0.checked_add(1).ok_or(StepError::TickOverflow)?);

        for input in inputs {
            let actor = &mut self.actors[input.seat.index()];
            let previous = actor.held.active();
            match input.action {
                InputAction::Press(direction) => actor.held.press(direction),
                InputAction::Release(direction) => actor.held.release(direction),
                InputAction::Clear => {
                    actor.held.clear();
                    actor.pending_turn = None;
                    actor.residual = 0;
                }
            }
            if actor.held.active() != previous {
                actor.pending_turn = None;
            }
            if actor.held.active().is_none() {
                actor.residual = 0;
            }
        }
        let traces = self
            .actors
            .each_mut()
            .map(|actor| movement::advance_actor(&self.world, actor));
        self.tick = next_tick;
        Ok(StepReport {
            tick: next_tick,
            traces,
        })
    }
}

impl From<ActorInit> for Actor {
    fn from(init: ActorInit) -> Self {
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
        }
    }
}

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
