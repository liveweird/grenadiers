use std::collections::BTreeSet;

use super::arena::Terrain;
use super::bombs::Bomb;
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
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    rules: RulesConfig,
    world: World,
    actors: [Actor; 2],
    tick: Tick,
    next_entity_id: u64,
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

    pub fn step(&mut self, inputs: &[SeatInput]) -> Result<StepReport, StepError> {
        if inputs.len() > MAX_INPUTS_PER_TICK {
            return Err(StepError::TooManyInputs {
                count: inputs.len(),
                maximum: MAX_INPUTS_PER_TICK,
            });
        }
        let next_tick = Tick(self.tick.0.checked_add(1).ok_or(StepError::TickOverflow)?);
        self.expire_flames(next_tick);

        let mut placement_requests = [false; 2];
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
                    placement_requests[input.seat.index()] = false;
                }
                InputAction::PlaceBomb => placement_requests[input.seat.index()] = true,
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
        self.resolve_placements(placement_requests, next_tick);
        self.resolve_explosions(next_tick);
        self.tick = next_tick;
        Ok(StepReport {
            tick: next_tick,
            traces,
        })
    }

    fn resolve_placements(&mut self, requests: [bool; 2], tick: Tick) {
        let mut targets = [None; 2];
        for seat in SeatId::ALL {
            let actor = &self.actors[seat.index()];
            let Some(cell) = actor.position.cell() else {
                continue;
            };
            let has_capacity = self.live_bomb_count(seat) < usize::from(actor.capacity);
            let cell_is_free = self.world.arena.terrain(cell) == Some(Terrain::Floor)
                && !self.world.arena.blocks().contains(&cell)
                && !self.world.bombs.values().any(|bomb| bomb.cell() == cell);
            if requests[seat.index()] && has_capacity && cell_is_free {
                targets[seat.index()] = Some(cell);
            }
        }
        if targets[0].is_some() && targets[0] == targets[1] {
            return;
        }
        for seat in SeatId::ALL {
            let Some(cell) = targets[seat.index()] else {
                continue;
            };
            let Some(detonates_at) = tick.0.checked_add(self.rules.bomb_fuse_ticks()).map(Tick)
            else {
                continue;
            };
            let Some(id) = self.next_bomb_id() else {
                continue;
            };
            let actor = &self.actors[seat.index()];
            self.insert_bomb(Bomb {
                id,
                cell,
                owner: actor.id,
                range: actor.range,
                placed_at: tick,
                detonates_at,
            });
        }
    }

    fn next_bomb_id(&self) -> Option<EntityId> {
        let mut candidate = self.next_entity_id;
        while let Ok(raw) = u32::try_from(candidate) {
            let id = EntityId(raw);
            if !self.actors.iter().any(|actor| actor.id == id)
                && !self.world.bombs.contains_key(&id)
            {
                return Some(id);
            }
            candidate += 1;
        }
        None
    }

    fn insert_bomb(&mut self, bomb: Bomb) {
        let id = bomb.id();
        let cell = bomb.cell();
        self.world.bombs.insert(id, bomb);
        self.next_entity_id = self.next_entity_id.max(u64::from(id.0) + 1);
        for actor in &mut self.actors {
            if movement::position_overlaps_cell(actor.position, cell) {
                actor.bomb_grace.insert(id);
            }
        }
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
        }
    }
}

#[cfg(test)]
#[path = "simulation_constructor_tests.rs"]
mod constructor_tests;
#[path = "simulation_explosions.rs"]
mod explosion_step;
#[cfg(test)]
#[path = "flame_tests.rs"]
mod flame_tests;
#[cfg(test)]
#[path = "placement_tests.rs"]
mod placement_tests;
#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
