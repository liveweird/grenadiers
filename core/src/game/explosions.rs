use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

use super::arena::Terrain;
use super::blast_geometry::{BlastBomb, BlastField, BlastSource, BlastVisit, visit_blast};
use super::bombs::BombTrigger;
use super::types::{CellId, EntityId, Tick};
use super::world::World;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct ExplosionPlan {
    pub(crate) detonated: BTreeSet<EntityId>,
    pub(crate) destroyed_mines: BTreeSet<EntityId>,
    pub(crate) flames: BTreeSet<CellId>,
    pub(crate) destroyed_blocks: BTreeSet<CellId>,
    pub(crate) ignited_oil: BTreeSet<CellId>,
}

#[cfg(test)]
pub(crate) fn plan(
    world: &World,
    tick: Tick,
    remote_targets: [Option<EntityId>; 2],
) -> ExplosionPlan {
    plan_with_contacts(world, tick, remote_targets, &BTreeSet::new())
}

pub(crate) fn plan_with_contacts(
    world: &World,
    tick: Tick,
    remote_targets: [Option<EntityId>; 2],
    contact_targets: &BTreeSet<EntityId>,
) -> ExplosionPlan {
    let bombs_by_cell: BTreeMap<_, _> = world
        .bombs
        .values()
        .map(|bomb| (bomb.cell(), bomb.id()))
        .collect();
    let field = WorldBlastField {
        world,
        bombs_by_cell: &bombs_by_cell,
    };
    let mut pending: BTreeSet<_> = world
        .bombs
        .values()
        .filter(|bomb| {
            let burning = world.has_fire(bomb.cell(), tick);
            (match bomb.trigger() {
                BombTrigger::Contact => contact_targets.contains(&bomb.id()),
                BombTrigger::Timed { detonates_at } => detonates_at <= tick,
                BombTrigger::Remote => remote_targets.contains(&Some(bomb.id())) || burning,
            }) || burning && !matches!(bomb.trigger(), BombTrigger::Contact)
        })
        .map(|bomb| bomb.id())
        .collect();
    let mut result = ExplosionPlan::default();
    result
        .ignited_oil
        .extend(world.oil.iter().filter_map(|(cell, state)| {
            (state == crate::game::OilState::Unlit && world.has_fire(cell, tick)).then_some(cell)
        }));
    result
        .destroyed_mines
        .extend(world.bombs.values().filter_map(|bomb| {
            (matches!(bomb.trigger(), BombTrigger::Contact)
                && !contact_targets.contains(&bomb.id())
                && world.has_fire(bomb.cell(), tick))
            .then_some(bomb.id())
        }));

    while let Some(id) = pending.pop_first() {
        if !result.detonated.insert(id) {
            continue;
        }
        let Some(bomb) = world.bombs.get(&id) else {
            continue;
        };
        let completed = visit_blast(
            &field,
            BlastSource {
                cell: bomb.cell(),
                range: bomb.range(),
                kind: bomb.blast(),
            },
            |visit| {
                apply_visit(visit, contact_targets, &mut pending, &mut result);
                ControlFlow::<()>::Continue(())
            },
        );
        debug_assert!(completed.is_continue());
    }
    result
}

fn apply_visit(
    visit: BlastVisit,
    contact_targets: &BTreeSet<EntityId>,
    pending: &mut BTreeSet<EntityId>,
    result: &mut ExplosionPlan,
) {
    result.flames.insert(visit.cell);
    if visit.unlit_oil {
        result.ignited_oil.insert(visit.cell);
    }
    if visit.soft_block {
        result.destroyed_blocks.insert(visit.cell);
    }
    if let Some(bomb) = visit.bomb {
        if bomb.contact {
            if contact_targets.contains(&bomb.id) {
                pending.insert(bomb.id);
            } else {
                result.destroyed_mines.insert(bomb.id);
            }
        } else {
            pending.insert(bomb.id);
        }
    }
}

struct WorldBlastField<'a> {
    world: &'a World,
    bombs_by_cell: &'a BTreeMap<CellId, EntityId>,
}

impl BlastField for WorldBlastField<'_> {
    fn is_floor(&self, cell: CellId) -> bool {
        self.world.arena.terrain(cell) == Some(Terrain::Floor)
    }

    fn has_soft_block(&self, cell: CellId) -> bool {
        self.world.arena.blocks().contains(&cell)
    }

    fn has_unlit_oil(&self, cell: CellId) -> bool {
        self.world.oil.unlit(cell)
    }

    fn bomb_at(&self, cell: CellId) -> Option<BlastBomb> {
        let id = *self.bombs_by_cell.get(&cell)?;
        let bomb = self
            .world
            .bombs
            .get(&id)
            .expect("bomb-cell index references existing bombs");
        Some(BlastBomb {
            id,
            contact: matches!(bomb.trigger(), BombTrigger::Contact),
        })
    }
}

#[cfg(test)]
#[path = "explosions_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "remote_explosion_tests.rs"]
mod remote_tests;
