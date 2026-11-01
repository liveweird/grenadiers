use std::collections::{BTreeMap, BTreeSet};

use crate::cpu::{
    BombKnowledge, MAX_MULTIPLAYER_OBSERVED_BOMBS, MemoryKey, Observation, ObservedBombTrigger,
    WorkBudget,
};
use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::{CellId, EntityId, MAX_OIL_BARRELS, OilBarrelState, Tick};
use crate::room::BoardSize;

use super::ForecastFailure;
use super::intervals::take;
use contact::bomb_matches;

const MAX_FLAME_CHANGES: usize = MAX_MULTIPLAYER_OBSERVED_BOMBS * ARENA_CELL_COUNT;
const _: () = assert!(MAX_FLAME_CHANGES == 6_240);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FlameChange {
    pub(super) at: Tick,
    pub(super) cell: CellId,
    pub(super) expires_at: Option<Tick>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForecastEvolution {
    key: MemoryKey,
    board: BoardSize,
    basis_tick: Tick,
    horizon_end: Tick,
    max_bombs: usize,
    bomb_owners: Vec<(EntityId, EntityId)>,
    capacity_ids: BTreeSet<EntityId>,
    max_flame_changes: usize,
    bomb_removals: Vec<(EntityId, Tick)>,
    block_removals: Vec<(CellId, Tick)>,
    oil_ignitions: Vec<(CellId, Tick)>,
    flame_changes: Vec<FlameChange>,
}

impl ForecastEvolution {
    pub(crate) const fn bomb_limit(&self) -> usize {
        self.max_bombs
    }

    pub(super) fn new(
        observation: &Observation,
        horizon_end: Tick,
    ) -> Result<Self, ForecastFailure> {
        let parts = observation.parts();
        let BombKnowledge::Visible(bombs) = &parts.bombs else {
            return Err(ForecastFailure::BombKnowledgeUnavailable);
        };
        if bombs.len() > crate::cpu::observed_bomb_capacity(parts.rules.version) {
            return Err(ForecastFailure::IntervalCapacity);
        }
        Ok(Self {
            key: MemoryKey {
                world: parts.world,
                cpu_seat: parts.recipient,
            },
            board: parts
                .arena
                .board()
                .ok_or(ForecastFailure::UnsupportedRules)?,
            basis_tick: parts.tick,
            horizon_end,
            max_bombs: crate::cpu::observed_bomb_capacity(parts.rules.version),
            bomb_owners: bombs.iter().map(|bomb| (bomb.id, bomb.owner)).collect(),
            capacity_ids: bombs
                .iter()
                .filter(|bomb| {
                    !matches!(parts.rules.version, 9 | 10)
                        || bomb.trigger != ObservedBombTrigger::Contact
                })
                .map(|bomb| bomb.id)
                .collect(),
            max_flame_changes: if matches!(parts.rules.version, 9 | 10) {
                ARENA_CELL_COUNT * ARENA_CELL_COUNT
            } else {
                MAX_FLAME_CHANGES
            },
            bomb_removals: Vec::new(),
            block_removals: Vec::new(),
            oil_ignitions: Vec::new(),
            flame_changes: Vec::new(),
        })
    }

    pub(super) fn record(
        &mut self,
        tick: Tick,
        removed_bombs: impl IntoIterator<Item = EntityId>,
        destroyed_blocks: impl IntoIterator<Item = CellId>,
        ignited_oil: impl IntoIterator<Item = CellId>,
        flames: impl IntoIterator<Item = FlameChange>,
        work: &mut WorkBudget,
    ) -> Result<(), ForecastFailure> {
        let removed = removed_bombs
            .into_iter()
            .filter(|id| self.bomb_owners.iter().any(|(known, _)| known == id))
            .collect::<Vec<_>>();
        let blocks = destroyed_blocks.into_iter().collect::<Vec<_>>();
        let oil = ignited_oil.into_iter().collect::<Vec<_>>();
        let flames = flames.into_iter().collect::<Vec<_>>();
        if self.bomb_removals.len() + removed.len() > self.max_bombs
            || self.block_removals.len() + blocks.len() > ARENA_CELL_COUNT
            || self.oil_ignitions.len() + oil.len() > MAX_OIL_BARRELS
            || self.flame_changes.len() + flames.len() > self.max_flame_changes
        {
            return Err(ForecastFailure::IntervalCapacity);
        }
        let units = removed.len() + blocks.len() + oil.len() + flames.len() + 1;
        take(
            work,
            u32::try_from(units).map_err(|_| ForecastFailure::WorkExhausted)?,
        )?;
        self.bomb_removals
            .extend(removed.into_iter().map(|id| (id, tick)));
        self.block_removals
            .extend(blocks.into_iter().map(|cell| (cell, tick)));
        self.oil_ignitions
            .extend(oil.into_iter().map(|cell| (cell, tick)));
        self.flame_changes.extend(flames);
        Ok(())
    }

    pub(super) fn add_hypothetical(
        &mut self,
        id: EntityId,
        owner: EntityId,
    ) -> Result<(), ForecastFailure> {
        if self.bomb_owners.len() >= self.max_bombs
            || self.bomb_owners.iter().any(|(known, _)| *known == id)
        {
            return Err(ForecastFailure::IntervalCapacity);
        }
        self.bomb_owners.push((id, owner));
        self.capacity_ids.insert(id);
        Ok(())
    }

    #[must_use]
    pub fn matches_scene(&self, basis: &Observation, current: &Observation) -> bool {
        self.header_matches(basis, current)
            && self.blocks_match(basis, current)
            && self.bombs_match(basis, current, None)
            && self.oil_matches(basis, current)
            && self.flames_match(basis, current)
    }

    pub(crate) fn matches_scene_with_hypothetical(
        &self,
        basis: &Observation,
        current: &Observation,
        hypothetical: EntityId,
        actual: EntityId,
    ) -> bool {
        self.header_matches(basis, current)
            && self.blocks_match(basis, current)
            && self.bombs_match(basis, current, Some((hypothetical, actual)))
            && self.oil_matches(basis, current)
            && self.flames_match(basis, current)
    }

    fn header_matches(&self, basis: &Observation, current: &Observation) -> bool {
        let old = basis.parts();
        let new = current.parts();
        let Some(elapsed) = new.tick.0.checked_sub(old.tick.0) else {
            return false;
        };
        if old.tick != self.basis_tick
            || new.tick >= self.horizon_end
            || (MemoryKey {
                world: old.world,
                cpu_seat: old.recipient,
            }) != self.key
            || old.arena.board() != Some(self.board)
            || old.world != new.world
            || old.recipient != new.recipient
            || old.rules.version != new.rules.version
            || old.rules.tick_rate != new.rules.tick_rate
            || old.rules.settings != new.rules.settings
            || u64::from(old.rules.running_updates).checked_add(elapsed)
                != Some(u64::from(new.rules.running_updates))
            || u64::from(old.rules.remaining_ticks).checked_sub(elapsed)
                != Some(u64::from(new.rules.remaining_ticks))
            || old.arena.width != new.arena.width
            || old.arena.height != new.arena.height
            || old.arena.terrain != new.arena.terrain
            || old.pickups != new.pickups
        {
            return false;
        }
        true
    }

    #[must_use]
    pub fn expected_live_bombs(&self, owner: EntityId, tick: Tick) -> Option<u16> {
        if tick < self.basis_tick || tick >= self.horizon_end {
            return None;
        }
        u16::try_from(
            self.bomb_owners
                .iter()
                .filter(|(id, bomb_owner)| {
                    *bomb_owner == owner
                        && self.capacity_ids.contains(id)
                        && !self.removed(*id, tick)
                })
                .count(),
        )
        .ok()
    }

    #[must_use]
    pub(crate) fn hypothetical_removed(&self, id: EntityId, tick: Tick) -> bool {
        self.removed(id, tick)
    }

    pub(super) fn block_removals(&self) -> impl Iterator<Item = (CellId, Tick)> + '_ {
        self.block_removals.iter().copied()
    }

    fn blocks_match(&self, basis: &Observation, current: &Observation) -> bool {
        let expected = basis
            .parts()
            .arena
            .blocks
            .iter()
            .copied()
            .filter(|cell| {
                !self
                    .block_removals
                    .iter()
                    .any(|(removed, at)| removed == cell && *at <= current.parts().tick)
            })
            .collect::<BTreeSet<_>>();
        current
            .parts()
            .arena
            .blocks
            .iter()
            .copied()
            .collect::<BTreeSet<_>>()
            == expected
    }

    fn bombs_match(
        &self,
        basis: &Observation,
        current: &Observation,
        mapping: Option<(EntityId, EntityId)>,
    ) -> bool {
        let (BombKnowledge::Visible(old), BombKnowledge::Visible(new)) =
            (&basis.parts().bombs, &current.parts().bombs)
        else {
            return false;
        };
        let elapsed = current.parts().tick.0 - basis.parts().tick.0;
        let mapped_alive = mapping
            .is_some_and(|(hypothetical, _)| !self.removed(hypothetical, current.parts().tick));
        let expected_count = old
            .iter()
            .filter(|bomb| !self.removed(bomb.id, current.parts().tick))
            .count()
            + usize::from(mapped_alive);
        new.len() == expected_count
            && old
                .iter()
                .filter(|bomb| !self.removed(bomb.id, current.parts().tick))
                .all(|bomb| {
                    new.iter()
                        .filter(|candidate| {
                            !mapping.is_some_and(|(_, actual)| candidate.id == actual)
                        })
                        .any(|candidate| bomb_matches(*bomb, *candidate, elapsed))
                })
            && mapping.is_none_or(|(hypothetical, actual)| {
                self.removed(hypothetical, current.parts().tick)
                    || new.iter().any(|bomb| bomb.id == actual)
            })
    }

    fn oil_matches(&self, basis: &Observation, current: &Observation) -> bool {
        let old = &basis.parts().oil;
        let new = &current.parts().oil;
        old.len() == new.len()
            && old.iter().all(|oil| {
                let expected = if self
                    .oil_ignitions
                    .iter()
                    .any(|(cell, at)| *cell == oil.cell && *at <= current.parts().tick)
                {
                    OilBarrelState::Burning
                } else {
                    oil.state
                };
                new.iter()
                    .any(|candidate| candidate.cell == oil.cell && candidate.state == expected)
            })
    }

    fn flames_match(&self, basis: &Observation, current: &Observation) -> bool {
        let tick = current.parts().tick;
        let mut expected = basis
            .parts()
            .flames
            .iter()
            .filter(|flame| {
                !basis
                    .parts()
                    .oil
                    .iter()
                    .any(|oil| oil.cell == flame.cell && oil.state == OilBarrelState::Burning)
            })
            .filter_map(|flame| {
                let expires = basis
                    .parts()
                    .tick
                    .0
                    .checked_add(u64::from(flame.remaining_ticks))
                    .map(Tick)?;
                (expires > tick).then_some((flame.cell, expires))
            })
            .collect::<BTreeMap<_, _>>();
        for change in self.flame_changes.iter().filter(|change| change.at <= tick) {
            if let Some(expires) = change.expires_at {
                if expires > tick {
                    expected
                        .entry(change.cell)
                        .and_modify(|old| *old = (*old).max(expires))
                        .or_insert(expires);
                }
            } else {
                expected.remove(&change.cell);
            }
        }
        expected.retain(|_, expires| *expires > tick);
        let actual = current
            .parts()
            .flames
            .iter()
            .filter(|flame| {
                !current
                    .parts()
                    .oil
                    .iter()
                    .any(|oil| oil.cell == flame.cell && oil.state == OilBarrelState::Burning)
            })
            .map(|flame| {
                tick.0
                    .checked_add(u64::from(flame.remaining_ticks))
                    .map(|expires| (flame.cell, Tick(expires)))
            })
            .collect::<Option<BTreeMap<_, _>>>();
        actual.is_some_and(|actual| actual == expected)
    }

    fn removed(&self, id: EntityId, tick: Tick) -> bool {
        self.bomb_removals
            .iter()
            .any(|(removed, at)| *removed == id && *at <= tick)
    }
}

#[path = "evolution_contact.rs"]
mod contact;

#[path = "evolution_continuation.rs"]
mod continuation;
