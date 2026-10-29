use std::collections::BTreeSet;

use crate::room::{BoardSize, LairConfig, LairSettings};

use super::BombTrigger;
use super::arena::Terrain;
use super::geometry::MovementTrace;
use super::input::Direction;
use super::movement;
use super::simulation::{Actor, StateError};
use super::types::{CellId, EntityId, Position, Tick};
use super::world::World;

pub const LAIR_LAYOUT_VERSION: u16 = 1;
pub const MAX_LAIRS: usize = 2;
pub const MAX_MOBS: usize = 16;
pub const MOB_UNITS_PER_SECOND: u16 = 1_536;

#[path = "lair_navigation.rs"]
mod navigation;
pub(crate) use navigation::{
    actor_position_contacts_mobs, actor_segment_contacts_mobs, actor_trace_contacts_mobs,
    traces_contact_actor,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LairView {
    pub cell: CellId,
    pub next_spawn_at: Tick,
    pub living_mobs: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MobView {
    pub id: EntityId,
    pub lair: u8,
    pub position: Position,
    pub heading: Direction,
    pub frozen: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Mob {
    pub(crate) id: EntityId,
    pub(crate) lair: u8,
    pub(crate) position: Position,
    pub(crate) heading: Direction,
    pub(super) residual: u16,
    pub(super) leg_origin: Position,
    pub(super) retreat_anchor: Option<Position>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Lair {
    cell: CellId,
    next_spawn_at: Tick,
    priority: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LairState {
    settings: LairSettings,
    lairs: [Lair; MAX_LAIRS],
    mobs: Vec<Mob>,
}

impl LairState {
    pub(crate) fn new(
        config: LairConfig,
        board: BoardSize,
        arena: &super::Arena,
        at: Tick,
    ) -> Result<Option<Self>, StateError> {
        let LairConfig::Enabled(settings) = config else {
            return Ok(None);
        };
        let cells = layout(board);
        for cell in cells {
            if arena.terrain(cell) != Some(Terrain::Wall) {
                return Err(StateError::InvalidLairCell(cell));
            }
        }
        let next = Tick(
            at.0.checked_add(settings.spawn_ticks())
                .ok_or(StateError::LairTimerOverflow)?,
        );
        Ok(Some(Self {
            settings,
            lairs: cells.map(|cell| Lair {
                cell,
                next_spawn_at: next,
                priority: 0,
            }),
            mobs: Vec::new(),
        }))
    }

    pub(crate) fn views(&self) -> [LairView; MAX_LAIRS] {
        std::array::from_fn(|index| LairView {
            cell: self.lairs[index].cell,
            next_spawn_at: self.lairs[index].next_spawn_at,
            living_mobs: self
                .mobs
                .iter()
                .filter(|mob| usize::from(mob.lair) == index)
                .count() as u8,
        })
    }

    pub(crate) fn mob_views(&self, frozen: bool) -> Vec<MobView> {
        self.mobs
            .iter()
            .map(|mob| MobView {
                id: mob.id,
                lair: mob.lair,
                position: mob.position,
                heading: mob.heading,
                frozen,
            })
            .collect()
    }

    pub(crate) fn due(&self, tick: Tick) -> bool {
        self.lairs.iter().any(|lair| lair.next_spawn_at <= tick)
    }

    pub(crate) fn contains_id(&self, id: EntityId) -> bool {
        self.mobs.iter().any(|mob| mob.id == id)
    }

    pub(crate) fn advance(
        &mut self,
        world: &World,
        actors: &[Actor],
        target_order: &[usize],
        frozen: bool,
        tick: Tick,
    ) -> Vec<MovementTrace> {
        let danger = super::explosions::bomb_danger_cells(world);
        let searches = navigation::target_searches(world, actors, target_order, tick, &danger);
        self.mobs
            .iter_mut()
            .map(|mob| navigation::advance_mob(mob, world, &danger, &searches, frozen, tick))
            .collect()
    }

    pub(crate) fn kill_on_fire(&mut self, world: &World) {
        self.mobs.retain(|mob| {
            !world
                .committed_fire_cells()
                .any(|cell| movement::position_overlaps_cell(mob.position, cell))
        });
    }

    pub(crate) fn mine_contacts(
        &self,
        world: &World,
        traces: &[MovementTrace],
    ) -> BTreeSet<EntityId> {
        world
            .bombs
            .values()
            .filter(|bomb| matches!(bomb.trigger(), BombTrigger::Contact))
            .filter(|bomb| {
                traces.iter().any(|trace| {
                    trace
                        .segments()
                        .any(|segment| segment.overlaps_cell(bomb.cell()))
                })
            })
            .map(|bomb| bomb.id())
            .collect()
    }

    pub(crate) fn spawn(
        &mut self,
        world: &World,
        actors: &[Actor],
        tick: Tick,
        ids: &mut impl Iterator<Item = EntityId>,
    ) -> Result<usize, ()> {
        let mut spawned = 0;
        for index in 0..MAX_LAIRS {
            if self.lairs[index].next_spawn_at > tick {
                continue;
            }
            self.lairs[index].next_spawn_at = Tick(
                tick.0
                    .checked_add(self.settings.spawn_ticks())
                    .expect("lair interval was preflight before mutation"),
            );
            let start = self.lairs[index].priority;
            self.lairs[index].priority = (start + 1) % 4;
            if self.mobs.len() >= MAX_MOBS
                || self
                    .mobs
                    .iter()
                    .filter(|mob| usize::from(mob.lair) == index)
                    .count()
                    >= usize::from(self.settings.mobs_per_lair())
            {
                continue;
            }
            let Some(cell) = (0..4)
                .map(|offset| adjacent(self.lairs[index].cell, (start + offset) % 4))
                .find(|&cell| spawn_is_clear(cell, world, actors, &self.mobs, tick))
            else {
                continue;
            };
            let Some(id) = ids.next() else { return Err(()) };
            self.mobs.push(Mob {
                id,
                lair: index as u8,
                position: Position::center(cell),
                heading: Direction::South,
                residual: 0,
                leg_origin: Position::center(cell),
                retreat_anchor: None,
            });
            spawned += 1;
        }
        Ok(spawned)
    }

    pub(crate) fn mobs(&self) -> &[Mob] {
        &self.mobs
    }

    pub(crate) fn overlaps_cell(&self, cell: CellId) -> bool {
        self.mobs
            .iter()
            .any(|mob| movement::position_overlaps_cell(mob.position, cell))
    }

    #[cfg(test)]
    pub(crate) fn insert_test_mob(
        &mut self,
        id: EntityId,
        lair: u8,
        position: Position,
        heading: Direction,
    ) {
        self.mobs.push(Mob {
            id,
            lair,
            position,
            heading,
            residual: 0,
            leg_origin: Position::center(position.cell().expect("test mob is on the board")),
            retreat_anchor: None,
        });
    }
}

fn layout(board: BoardSize) -> [CellId; MAX_LAIRS] {
    match board {
        BoardSize::FifteenByThirteen => [CellId::new(6, 6), CellId::new(8, 6)],
        BoardSize::ThirteenByEleven => [CellId::new(6, 4), CellId::new(6, 6)],
        BoardSize::ElevenByNine => [CellId::new(4, 4), CellId::new(6, 4)],
    }
}

fn adjacent(cell: CellId, direction: u8) -> CellId {
    match direction {
        0 => CellId::new(cell.column, cell.row - 1),
        1 => CellId::new(cell.column + 1, cell.row),
        2 => CellId::new(cell.column, cell.row + 1),
        _ => CellId::new(cell.column - 1, cell.row),
    }
}

fn spawn_is_clear(cell: CellId, world: &World, actors: &[Actor], mobs: &[Mob], tick: Tick) -> bool {
    traversable(cell, world, &BTreeSet::new(), tick)
        && world.oil.state(cell).is_none()
        && !actors
            .iter()
            .any(|actor| actor.alive() && movement::position_overlaps_cell(actor.position(), cell))
        && !mobs
            .iter()
            .any(|mob| movement::position_overlaps_cell(mob.position, cell))
}

fn traversable(cell: CellId, world: &World, danger: &BTreeSet<CellId>, tick: Tick) -> bool {
    world.arena.terrain(cell) == Some(Terrain::Floor)
        && !world.arena.blocks().contains(&cell)
        && !world.bombs.values().any(|bomb| bomb.cell() == cell)
        && !world.oil.unlit(cell)
        && !world.has_fire(cell, tick)
        && !danger.contains(&cell)
}
