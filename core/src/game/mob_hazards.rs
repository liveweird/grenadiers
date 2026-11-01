use std::collections::{BTreeMap, BTreeSet};
use std::ops::ControlFlow;

use super::bombs::BombTrigger;
use super::types::{CellId, EntityId, Tick};
use super::world::World;

pub(crate) const MOB_NAVIGATION_WORK: usize = 8_192;

#[derive(Debug)]
pub(crate) struct WorkBudget {
    remaining: usize,
}

impl WorkBudget {
    pub(crate) const fn new() -> Self {
        Self {
            remaining: MOB_NAVIGATION_WORK,
        }
    }

    pub(crate) fn charge(&mut self) -> Option<()> {
        self.remaining = self.remaining.checked_sub(1)?;
        Some(())
    }
}

#[derive(Debug)]
pub(crate) struct TimedMobHazards {
    deadlines: BTreeMap<CellId, Tick>,
}

impl TimedMobHazards {
    pub(crate) fn build(world: &World, tick: Tick, budget: &mut WorkBudget) -> Option<Self> {
        if world.bombs.values().any(|bomb| {
            bomb.motion().is_some()
                || !matches!(bomb.trigger(), BombTrigger::Timed { .. })
                || world.has_fire(bomb.cell(), tick)
        }) {
            return None;
        }
        if world.oil.iter().any(|(_, state)| {
            matches!(
                state,
                super::oil::OilState::Concealed | super::oil::OilState::Pending
            )
        }) {
            return Self::build_uncertain(world, budget);
        }
        let ids = world.bombs.keys().copied().collect::<Vec<_>>();
        let mut deadlines = BTreeMap::new();
        for &id in &ids {
            budget.charge()?;
            let bomb = world.bombs.get(&id)?;
            let BombTrigger::Timed { detonates_at } = bomb.trigger() else {
                return None;
            };
            deadlines.insert(id, detonates_at);
        }
        let mut removals =
            world
                .arena
                .blocks()
                .iter()
                .copied()
                .chain(world.oil.iter().filter_map(|(cell, state)| {
                    (state == super::oil::OilState::Unlit).then_some(cell)
                }))
                .map(|cell| {
                    let removal = world.has_fire(cell, tick).then_some(tick);
                    (cell, removal)
                })
                .collect::<BTreeMap<_, Option<Tick>>>();

        let mut changed = true;
        while changed {
            changed = false;
            for &source in &ids {
                let phase = deadlines.get(&source).copied()?;
                let bombs = phase_bombs(world, &ids, &deadlines, phase, budget)?;
                let blockers = phase_blockers(&removals, phase, budget)?;
                let mut bomb_hits = Vec::new();
                let mut blocker_hits = Vec::new();
                let completed = super::explosions::visit_potential_timed_blast(
                    world,
                    *world.bombs.get(&source)?,
                    &bombs,
                    &blockers,
                    |visit| {
                        if budget.charge().is_none() {
                            return ControlFlow::Break(());
                        }
                        if visit.soft_block {
                            blocker_hits.push(visit.cell);
                        }
                        if let Some(bomb) = visit.bomb
                            && bomb.id != source
                        {
                            bomb_hits.push(bomb.id);
                        }
                        ControlFlow::Continue(())
                    },
                );
                if completed.is_break() {
                    return None;
                }
                for target in bomb_hits {
                    budget.charge()?;
                    let Some(target_deadline) = deadlines.get_mut(&target) else {
                        continue;
                    };
                    if phase < *target_deadline {
                        *target_deadline = phase;
                        changed = true;
                    }
                }
                for cell in blocker_hits {
                    budget.charge()?;
                    let removal = removals.get_mut(&cell)?;
                    if removal.is_none_or(|current| phase < current) {
                        *removal = Some(phase);
                        changed = true;
                    }
                }
            }
        }

        let mut cell_deadlines = BTreeMap::new();
        for &source in &ids {
            let phase = deadlines.get(&source).copied()?;
            let bombs = phase_bombs(world, &ids, &deadlines, phase, budget)?;
            let blockers = phase_blockers(&removals, phase, budget)?;
            let completed = super::explosions::visit_potential_timed_blast(
                world,
                *world.bombs.get(&source)?,
                &bombs,
                &blockers,
                |visit| {
                    if budget.charge().is_none() {
                        return ControlFlow::Break(());
                    }
                    cell_deadlines
                        .entry(visit.cell)
                        .and_modify(|current: &mut Tick| *current = (*current).min(phase))
                        .or_insert(phase);
                    ControlFlow::Continue(())
                },
            );
            if completed.is_break() {
                return None;
            }
        }
        Some(Self {
            deadlines: cell_deadlines,
        })
    }

    pub(crate) fn deadline(&self, cell: CellId) -> Option<Tick> {
        self.deadlines.get(&cell).copied()
    }

    pub(crate) fn contains(&self, cell: CellId) -> bool {
        self.deadlines.contains_key(&cell)
    }

    pub(crate) fn cells(&self) -> impl Iterator<Item = CellId> + '_ {
        self.deadlines.keys().copied()
    }

    fn build_uncertain(world: &World, budget: &mut WorkBudget) -> Option<Self> {
        let ids = world.bombs.keys().copied().collect::<Vec<_>>();
        let mut deadlines = BTreeMap::new();
        let mut footprints = BTreeMap::<EntityId, BTreeSet<CellId>>::new();
        let no_bombs = BTreeMap::new();
        let no_blockers = BTreeSet::new();
        for &id in &ids {
            budget.charge()?;
            let bomb = *world.bombs.get(&id)?;
            let BombTrigger::Timed { detonates_at } = bomb.trigger() else {
                return None;
            };
            deadlines.insert(id, detonates_at);
            let cells = footprints.entry(id).or_default();
            let completed = super::explosions::visit_potential_timed_blast(
                world,
                bomb,
                &no_bombs,
                &no_blockers,
                |visit| {
                    if budget.charge().is_none() {
                        return ControlFlow::Break(());
                    }
                    cells.insert(visit.cell);
                    ControlFlow::Continue(())
                },
            );
            if completed.is_break() {
                return None;
            }
        }
        relax_full_potential_deadlines(world, &ids, &footprints, &mut deadlines, budget)?;
        let mut cell_deadlines = BTreeMap::new();
        for &id in &ids {
            let deadline = deadlines.get(&id).copied()?;
            for &cell in footprints.get(&id)? {
                budget.charge()?;
                cell_deadlines
                    .entry(cell)
                    .and_modify(|current: &mut Tick| *current = (*current).min(deadline))
                    .or_insert(deadline);
            }
        }
        Some(Self {
            deadlines: cell_deadlines,
        })
    }
}

fn relax_full_potential_deadlines(
    world: &World,
    ids: &[EntityId],
    footprints: &BTreeMap<EntityId, BTreeSet<CellId>>,
    deadlines: &mut BTreeMap<EntityId, Tick>,
    budget: &mut WorkBudget,
) -> Option<()> {
    let mut changed = true;
    while changed {
        changed = false;
        for &source in ids {
            let source_deadline = deadlines.get(&source).copied()?;
            for &target in ids {
                budget.charge()?;
                if source == target
                    || !footprints
                        .get(&source)?
                        .contains(&world.bombs.get(&target)?.cell())
                {
                    continue;
                }
                let target_deadline = deadlines.get_mut(&target)?;
                if source_deadline < *target_deadline {
                    *target_deadline = source_deadline;
                    changed = true;
                }
            }
        }
    }
    Some(())
}

fn phase_bombs(
    world: &World,
    ids: &[EntityId],
    deadlines: &BTreeMap<EntityId, Tick>,
    phase: Tick,
    budget: &mut WorkBudget,
) -> Option<BTreeMap<CellId, EntityId>> {
    let mut result = BTreeMap::new();
    for &id in ids {
        budget.charge()?;
        if deadlines
            .get(&id)
            .is_some_and(|deadline| *deadline >= phase)
        {
            result.insert(world.bombs.get(&id)?.cell(), id);
        }
    }
    Some(result)
}

fn phase_blockers(
    removals: &BTreeMap<CellId, Option<Tick>>,
    phase: Tick,
    budget: &mut WorkBudget,
) -> Option<BTreeSet<CellId>> {
    let mut result = BTreeSet::new();
    for (&cell, &removal) in removals {
        budget.charge()?;
        if removal.is_none_or(|deadline| deadline >= phase) {
            result.insert(cell);
        }
    }
    Some(result)
}
