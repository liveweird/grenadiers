use std::collections::{BTreeMap, BTreeSet};

use super::arena::Terrain;
use super::bombs::{BombMotion, BombTrigger};
use super::input::Direction;
use super::movement::position_overlaps_cell;
use super::simulation::Actor;
use super::types::{CellId, EntityId, Tick};
use super::world::World;

pub(crate) struct MotionPhase {
    pub(crate) moving_at_start: BTreeSet<EntityId>,
    pub(crate) moved: BTreeSet<EntityId>,
}

pub(crate) fn advance(world: &mut World, actors: &[Actor; 2], tick: Tick) -> MotionPhase {
    let moving_at_start: BTreeSet<_> = world
        .bombs
        .values()
        .filter(|bomb| bomb.motion().is_some())
        .map(|bomb| bomb.id())
        .collect();
    let mut proposals = BTreeMap::new();
    let mut stop = BTreeSet::new();

    for id in &moving_at_start {
        let bomb = world.bombs[id];
        let motion = bomb.motion().expect("moving set was captured from motion");
        let expired =
            matches!(bomb.trigger(), BombTrigger::Timed { detonates_at } if detonates_at <= tick);
        let burning = world.has_fire(bomb.cell(), tick);
        if expired || burning {
            continue;
        }
        if motion.countdown > 0 {
            world
                .bombs
                .get_mut(id)
                .expect("captured bomb exists")
                .set_motion(Some(BombMotion {
                    countdown: motion.countdown - 1,
                    ..motion
                }));
            continue;
        }
        let Some(target) = adjacent(bomb.cell(), motion.direction) else {
            stop.insert(*id);
            continue;
        };
        if blocked(world, actors, *id, bomb.cell(), target) {
            stop.insert(*id);
        } else {
            proposals.insert(*id, (target, motion.direction));
        }
    }

    let mut claims = BTreeMap::<CellId, usize>::new();
    for (target, _) in proposals.values() {
        *claims.entry(*target).or_default() += 1;
    }
    let conflicted: Vec<_> = proposals
        .iter()
        .filter_map(|(id, (target, _))| (claims[target] > 1).then_some(*id))
        .collect();
    for id in conflicted {
        proposals.remove(&id);
        stop.insert(id);
    }

    for id in &stop {
        world
            .bombs
            .get_mut(id)
            .expect("captured bomb exists")
            .set_motion(None);
    }
    let moved: BTreeSet<_> = proposals.keys().copied().collect();
    for (id, (target, direction)) in proposals {
        let bomb = world.bombs.get_mut(&id).expect("proposal bomb exists");
        bomb.set_cell(target);
        bomb.set_motion(Some(BombMotion {
            direction,
            countdown: 9,
        }));
    }
    MotionPhase {
        moving_at_start,
        moved,
    }
}

fn blocked(
    world: &World,
    actors: &[Actor; 2],
    id: EntityId,
    source: CellId,
    target: CellId,
) -> bool {
    world.arena.terrain(target) != Some(Terrain::Floor)
        || world.arena.blocks().contains(&target)
        || world.oil.unlit(target)
        || world
            .bombs
            .values()
            .any(|bomb| bomb.id() != id && bomb.cell() == target)
        || actors.iter().any(|actor| {
            actor.alive()
                && (position_overlaps_cell(actor.position(), source)
                    || position_overlaps_cell(actor.position(), target))
        })
}

fn adjacent(cell: CellId, direction: Direction) -> Option<CellId> {
    match direction {
        Direction::North => cell
            .row
            .checked_sub(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::East => cell
            .column
            .checked_add(1)
            .map(|column| CellId::new(column, cell.row)),
        Direction::South => cell
            .row
            .checked_add(1)
            .map(|row| CellId::new(cell.column, row)),
        Direction::West => cell
            .column
            .checked_sub(1)
            .map(|column| CellId::new(column, cell.row)),
    }
}
