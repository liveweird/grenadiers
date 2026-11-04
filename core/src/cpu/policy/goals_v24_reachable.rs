use std::collections::VecDeque;

use crate::cpu::{Observation, WorkBudget};
use crate::game::CellId;
use crate::game::arena::ARENA_CELL_COUNT;

use crate::cpu::policy::movement::{STEPS, offset};

use super::{DistanceMap, FirstStepMap, cell_index};

pub(in crate::cpu::policy) fn reachable(
    observation: &Observation,
    current: CellId,
    map: &super::super::SearchMap,
    work: &mut WorkBudget,
) -> Option<(DistanceMap, FirstStepMap)> {
    let mut distance: [Option<u16>; ARENA_CELL_COUNT] = [None; ARENA_CELL_COUNT];
    let mut first_step = [None; ARENA_CELL_COUNT];
    let start = cell_index(observation, current)?;
    distance[start] = Some(0);
    first_step[start] = Some(current);
    let mut queue = VecDeque::from([current]);
    while let Some(cell) = queue.pop_front() {
        work.try_take(1).ok()?;
        let index = cell_index(observation, cell)?;
        let next_distance = distance[index]?.saturating_add(1);
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            let Some(next_index) = cell_index(observation, next) else {
                continue;
            };
            if distance[next_index].is_none() && map.transitable(observation, next) {
                distance[next_index] = Some(next_distance);
                first_step[next_index] = Some(if cell == current {
                    next
                } else {
                    first_step[index]?
                });
                queue.push_back(next);
            }
        }
    }
    Some((distance, first_step))
}
