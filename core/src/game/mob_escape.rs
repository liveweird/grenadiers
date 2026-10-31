use std::collections::{BTreeSet, VecDeque};

use super::arena::Terrain;
use super::lairs::MOB_UNITS_PER_SECOND;
use super::mob_hazards::{TimedMobHazards, WorkBudget};
use super::types::{CellId, HALF_FOOTPRINT, Position, TICKS_PER_SECOND, Tick, UNITS_PER_CELL};
use super::world::World;

#[derive(Clone, Copy)]
struct SearchNode {
    cell: CellId,
    steps: u16,
    first_direction: u8,
}

#[derive(Clone, Copy)]
pub(crate) struct EscapeEnvironment<'a> {
    pub(crate) world: &'a World,
    pub(crate) spike_pits: Option<&'a BTreeSet<CellId>>,
    pub(crate) hazards: &'a TimedMobHazards,
    pub(crate) tick: Tick,
}

pub(crate) fn escape_direction(
    environment: EscapeEnvironment<'_>,
    budget: &mut WorkBudget,
    position: Position,
    residual: u16,
) -> Option<u8> {
    let start = position.cell()?;
    if position != Position::center(start)
        || environment.hazards.deadline(start).is_none()
        || !clears_before_deadline(environment.hazards, start, 0, residual, environment.tick)?
    {
        return None;
    }

    let mut visited = vec![false; environment.world.arena.cell_count()];
    visited[index(environment.world, start)] = true;
    let mut pending = VecDeque::new();
    for direction in 0..4 {
        enqueue(
            environment,
            budget,
            residual,
            SearchNode {
                cell: super::lairs::adjacent(start, direction),
                steps: 1,
                first_direction: direction,
            },
            &mut visited,
            &mut pending,
        )?;
    }

    while let Some(node) = pending.pop_front() {
        budget.charge()?;
        if !environment.hazards.contains(node.cell) {
            return Some(node.first_direction);
        }
        for direction in 0..4 {
            enqueue(
                environment,
                budget,
                residual,
                SearchNode {
                    cell: super::lairs::adjacent(node.cell, direction),
                    steps: node.steps.checked_add(1)?,
                    first_direction: node.first_direction,
                },
                &mut visited,
                &mut pending,
            )?;
        }
    }
    None
}

fn enqueue(
    environment: EscapeEnvironment<'_>,
    budget: &mut WorkBudget,
    residual: u16,
    node: SearchNode,
    visited: &mut [bool],
    pending: &mut VecDeque<SearchNode>,
) -> Option<()> {
    budget.charge()?;
    if !traversable(
        node.cell,
        environment.world,
        environment.spike_pits,
        environment.tick,
    ) {
        return Some(());
    }
    let cell_index = index(environment.world, node.cell);
    if visited[cell_index] {
        return Some(());
    }
    if environment.hazards.deadline(node.cell).is_some()
        && !clears_before_deadline(
            environment.hazards,
            node.cell,
            node.steps,
            residual,
            environment.tick,
        )?
    {
        return Some(());
    }
    visited[cell_index] = true;
    pending.push_back(node);
    Some(())
}

fn clears_before_deadline(
    hazards: &TimedMobHazards,
    cell: CellId,
    steps: u16,
    residual: u16,
    tick: Tick,
) -> Option<bool> {
    let deadline = hazards.deadline(cell)?;
    let distance = i64::from(steps)
        .checked_mul(i64::from(UNITS_PER_CELL))?
        .checked_add(i64::from(UNITS_PER_CELL / 2 + HALF_FOOTPRINT))?;
    let numerator = distance
        .checked_mul(i64::from(TICKS_PER_SECOND))?
        .checked_sub(i64::from(residual))?
        .max(0);
    let speed = i64::from(MOB_UNITS_PER_SECOND);
    let travel_ticks = numerator.checked_add(speed - 1)?.checked_div(speed)?;
    let elapsed_after_current_tick = u64::try_from(travel_ticks).ok()?.checked_sub(1)?;
    let arrival = tick.0.checked_add(elapsed_after_current_tick)?;
    Some(arrival <= deadline.0)
}

fn traversable(
    cell: CellId,
    world: &World,
    spike_pits: Option<&BTreeSet<CellId>>,
    tick: Tick,
) -> bool {
    world.arena.terrain(cell) == Some(Terrain::Floor)
        && !world.arena.blocks().contains(&cell)
        && !world.bombs.values().any(|bomb| bomb.cell() == cell)
        && !world.oil.unlit(cell)
        && world.oil.state(cell) != Some(super::oil::OilState::Pending)
        && !world.has_fire(cell, tick)
        && !spike_pits.is_some_and(|pits| pits.contains(&cell))
}

fn index(world: &World, cell: CellId) -> usize {
    usize::from(cell.row) * usize::from(world.arena.width()) + usize::from(cell.column)
}
