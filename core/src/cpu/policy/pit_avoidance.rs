use crate::cpu::{Observation, WorkBudget};
use crate::game::CellId;

use super::goals::{SearchMap, cell_index};

pub(super) struct PitMask([bool; crate::game::arena::ARENA_CELL_COUNT]);

impl PitMask {
    pub(super) const EMPTY: Self = Self([false; crate::game::arena::ARENA_CELL_COUNT]);

    pub(super) fn contains(&self, observation: &Observation, cell: CellId) -> bool {
        cell_index(observation, cell).is_some_and(|index| self.0[index])
    }
}

pub(super) fn block_search_map(
    observation: &Observation,
    map: &mut SearchMap,
    work: &mut WorkBudget,
) -> Option<PitMask> {
    let pits = observation.spike_pits().map_or(&[][..], |pits| pits.pits());
    work.try_take(u32::try_from(pits.len()).ok()?).ok()?;
    let mut mask = PitMask::EMPTY;
    for pit in pits {
        let index = cell_index(observation, pit.cell)?;
        map.forbidden[index] = true;
        mask.0[index] = true;
    }
    Some(mask)
}

pub(super) fn retained_step_allowed(
    observation: &Observation,
    cell: CellId,
    work: &mut WorkBudget,
) -> bool {
    let pits = observation.spike_pits().map_or(&[][..], |pits| pits.pits());
    work.try_take(u32::try_from(pits.len()).unwrap_or(u32::MAX))
        .is_ok()
        && !pits.iter().any(|pit| pit.cell == cell)
}
