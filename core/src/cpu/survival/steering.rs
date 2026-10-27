use std::collections::{BTreeSet, VecDeque};

use crate::cpu::forecast::HazardForecast;
use crate::cpu::policy::ControlPlan;
use crate::cpu::{Observation, WorkBudget};
use crate::game::geometry::position_overlaps_cell;
use crate::game::{CellId, Position, Terrain};

/// Static guidance only. Every selected action still requires its own pulse
/// certificate; this search grants no timed-route or escape guarantee.
pub(super) fn target(
    observation: &Observation,
    forecast: &HazardForecast,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<CellId> {
    let position = super::compatibility::actor(observation)?.position;
    let current = position.cell()?;
    work.try_take(crate::game::arena::ARENA_CELL_COUNT as u32)
        .ok()?;
    let threatened = forecast.cells().iter().enumerate().any(|(index, hazards)| {
        let width = usize::from(observation.parts().arena.width);
        let cell = CellId::new((index % width) as u16, (index / width) as u16);
        position_overlaps_cell(position, cell) && !hazards.lethal.is_empty()
    });
    if !threatened {
        return ordinary
            .next_memory
            .route()
            .iter()
            .flatten()
            .copied()
            .find(|cell| *cell != current)
            .or(Some(current));
    }
    let mut queue = VecDeque::from([(current, current)]);
    let mut visited = BTreeSet::from([current]);
    while let Some((cell, first)) = queue.pop_front() {
        work.try_take(1).ok()?;
        let reached = cell != current && forecast.cell(cell)?.lethal.is_empty();
        if reached {
            return Some(first);
        }
        for next in adjacent(cell) {
            work.try_take(1).ok()?;
            if visited.insert(next) && traversable(observation, next) {
                queue.push_back((next, if cell == current { next } else { first }));
            }
        }
    }
    None
}

fn adjacent(cell: CellId) -> impl Iterator<Item = CellId> {
    [(0, -1), (1, 0), (0, 1), (-1, 0)]
        .into_iter()
        .filter_map(move |(dc, dr)| {
            Some(CellId::new(
                u16::try_from(i32::from(cell.column) + dc).ok()?,
                u16::try_from(i32::from(cell.row) + dr).ok()?,
            ))
        })
}

fn traversable(observation: &Observation, cell: CellId) -> bool {
    let parts = observation.parts();
    if cell.column >= parts.arena.width || cell.row >= parts.arena.height {
        return false;
    }
    let index = usize::from(cell.row) * usize::from(parts.arena.width) + usize::from(cell.column);
    parts.arena.terrain.get(index) == Some(&Terrain::Floor)
        && (crate::cpu::capabilities::can_pass_known_soft_blocks(observation)
            || !parts.arena.blocks.contains(&cell))
        && !parts.oil.iter().any(|oil| oil.cell == cell)
        && !parts.pickups.iter().any(|pickup| pickup.cell == cell)
}

pub(super) fn score(lease: &crate::cpu::routing::PulseLease, target: CellId) -> u32 {
    let bounds = lease.pose_after(lease.hold_ticks());
    let target = Position::center(target);
    bounds
        .min()
        .x
        .abs_diff(target.x)
        .max(bounds.max().x.abs_diff(target.x))
        + bounds
            .min()
            .y
            .abs_diff(target.y)
            .max(bounds.max().y.abs_diff(target.y))
}
