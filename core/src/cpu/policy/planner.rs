use std::collections::VecDeque;

use crate::cpu::{
    BombKnowledge, CpuRng, Intent, ManualIntent, MemoryKey, Observation, ObservedActor, WorkBudget,
};
use crate::game::CellId;
use crate::game::types::UNITS_PER_CELL;

use super::execution::follow_committed_escape;
use super::hazards::{immediately_safe, manhattan, planting_supported, unsupported_control_effect};
use super::movement::{
    STEPS, direction, hold_ticks, input_direction, offset, pulse_minimum_distance,
};
use super::{ControlPlan, MAX_HOLD_TICKS, MAX_ROUTE_CELLS, PolicyError, UtilityMemory};

const PER_PULSE_IDLE_TICKS: u64 = 10;
const CENTER_TOLERANCE: u64 = 64;

pub fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    let parts = observation.parts();
    let key = MemoryKey {
        world: parts.world,
        cpu_seat: parts.recipient,
    };
    if memory.key() != key || rng.key() != key {
        return Err(PolicyError::IdentityMismatch);
    }
    if let Some(committed) = follow_committed_escape(observation, memory)? {
        return Ok(committed);
    }
    let actor = parts
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
            _ => None,
        })
        .ok_or(PolicyError::InvalidObservation)?;
    if !actor.alive
        || actor.voluntary_actions_frozen
        || unsupported_control_effect(actor)
        || matches!(&parts.bombs, BombKnowledge::Unavailable)
    {
        return Ok(ControlPlan::stop(*memory));
    }
    let current = actor
        .position
        .cell()
        .ok_or(PolicyError::InvalidObservation)?;
    if work.try_take(1).is_err() {
        return Ok(ControlPlan::stop(*memory));
    }

    if memory.planted_origin().is_none()
        && let Some((next, advanced)) = memory.next(current, actor.position)
        && immediately_safe(observation, next)
    {
        return Ok(movement_plan(actor, current, next, advanced));
    }

    let can_prepare_bomb = adjacent_block(observation, current) && planting_supported(observation);
    let center_threshold = placement_center_threshold(observation);
    if can_prepare_bomb
        && !centered_for_placement(observation, actor, current)
        && let Some(desired) = alignment_direction(actor, current, center_threshold)
    {
        return Ok(ControlPlan {
            intent: Intent::moving(input_direction(actor, desired)),
            hold_ticks: hold_ticks(actor, current, desired),
            next_memory: UtilityMemory::new(key),
        });
    }

    if can_prepare_bomb
        && centered_for_placement(observation, actor, current)
        && !parts.pickups.iter().any(|pickup| pickup.cell == current)
        && let Some(route) = escape_route(observation, current, actor.bomb_range, work)
        && escape_before_fuse(
            actor,
            route.len().saturating_sub(1),
            parts.rules.settings.bomb_fuse_ticks(),
        )
    {
        let memory = UtilityMemory::from_route(key, &route, true);
        let next = route[1];
        let desired = direction(current, next).ok_or(PolicyError::InvalidObservation)?;
        return Ok(ControlPlan {
            intent: Intent::moving(input_direction(actor, desired)).with_manual(ManualIntent::Bomb),
            hold_ticks: hold_ticks(actor, next, desired),
            next_memory: memory,
        });
    }

    navigate(observation, actor, current, key, rng, work)
}

pub(super) fn alignment_direction(
    actor: &crate::cpu::VisibleActor,
    cell: CellId,
    threshold: u32,
) -> Option<crate::game::Direction> {
    let center = crate::game::Position::center(cell);
    let x_difference = actor.position.x.abs_diff(center.x);
    let y_difference = actor.position.y.abs_diff(center.y);
    let x_violates = x_difference > threshold;
    let y_violates = y_difference > threshold;
    let align_x = x_violates
        && (!y_violates
            || matches!(
                actor.heading,
                crate::game::Direction::East | crate::game::Direction::West
            ));
    if !x_violates && !y_violates {
        None
    } else if align_x && actor.position.x > center.x {
        Some(crate::game::Direction::West)
    } else if align_x && actor.position.x < center.x {
        Some(crate::game::Direction::East)
    } else if actor.position.y > center.y {
        Some(crate::game::Direction::North)
    } else if actor.position.y < center.y {
        Some(crate::game::Direction::South)
    } else {
        None
    }
}

fn centered_for_placement(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    cell: CellId,
) -> bool {
    let center = crate::game::Position::center(cell);
    let threshold = placement_center_threshold(observation);
    actor.position.x.abs_diff(center.x) <= threshold
        && actor.position.y.abs_diff(center.y) <= threshold
}

fn placement_center_threshold(observation: &Observation) -> u32 {
    observation
        .parts()
        .rules
        .settings
        .turn_assist_units()
        .min(64) as u32
}

fn navigate(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    current: CellId,
    key: MemoryKey,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    let target = goal(observation, current);
    let mut choices = Vec::new();
    for (desired, dc, dr) in STEPS {
        if work.try_take(1).is_err() {
            return Ok(ControlPlan::stop(UtilityMemory::new(key)));
        }
        if let Some(cell) = offset(current, dc, dr)
            && immediately_safe(observation, cell)
        {
            choices.push((
                manhattan(cell, target),
                cell.row,
                cell.column,
                desired,
                cell,
            ));
        }
    }
    if choices.is_empty() {
        return Ok(ControlPlan::stop(UtilityMemory::new(key)));
    }
    choices.sort_unstable_by_key(|choice| (choice.0, choice.1, choice.2));
    let tied = choices
        .iter()
        .take_while(|choice| choice.0 == choices[0].0)
        .count();
    let chosen = if tied > 1 {
        rng.bounded_u32(tied as u32).unwrap_or(0) as usize
    } else {
        0
    };
    let (_, _, _, desired, next) = choices[chosen];
    Ok(ControlPlan {
        intent: Intent::moving(input_direction(actor, desired)),
        hold_ticks: hold_ticks(actor, next, desired),
        next_memory: UtilityMemory::from_route(key, &[next], false),
    })
}

pub(crate) fn goal(observation: &Observation, current: CellId) -> CellId {
    if let Some(pickup) = observation.parts().pickups.iter().min_by_key(|pickup| {
        (
            manhattan(current, pickup.cell),
            pickup.cell.row,
            pickup.cell.column,
        )
    }) {
        return pickup.cell;
    }
    if let Some(opening) = observation
        .parts()
        .arena
        .blocks
        .iter()
        .flat_map(|block| {
            STEPS
                .iter()
                .filter_map(move |(_, dc, dr)| offset(*block, *dc, *dr))
        })
        .filter(|cell| immediately_safe(observation, *cell))
        .min_by_key(|cell| (manhattan(current, *cell), cell.row, cell.column))
    {
        return opening;
    }
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor)
                if actor.seat != observation.parts().recipient && actor.alive =>
            {
                actor.position.cell()
            }
            _ => None,
        })
        .unwrap_or(current)
}

fn escape_route(
    observation: &Observation,
    origin: CellId,
    range: u16,
    work: &mut WorkBudget,
) -> Option<Vec<CellId>> {
    let mut queue = VecDeque::from([vec![origin]]);
    while let Some(path) = queue.pop_front() {
        if work.try_take(1).is_err() {
            return None;
        }
        let cell = *path.last()?;
        if path.len() >= 3 && refuge_outside_cross(origin, cell, range) {
            return Some(path);
        }
        if path.len() == MAX_ROUTE_CELLS {
            continue;
        }
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            if !path.contains(&next)
                && next != origin
                && immediately_safe(observation, next)
                && !observation
                    .parts()
                    .pickups
                    .iter()
                    .any(|pickup| pickup.cell == next)
            {
                let mut extended = path.clone();
                extended.push(next);
                queue.push_back(extended);
            }
        }
    }
    None
}

fn refuge_outside_cross(origin: CellId, refuge: CellId, _range: u16) -> bool {
    refuge.column != origin.column && refuge.row != origin.row
}

fn escape_before_fuse(actor: &crate::cpu::VisibleActor, steps: usize, fuse: u64) -> bool {
    let per_pulse = pulse_minimum_distance(actor, MAX_HOLD_TICKS).max(1);
    // The actor may be stationary anywhere in its containing cell. Reserve a
    // half-cell approach plus the complete next-cell traversal and center window.
    let first =
        u64::try_from(UNITS_PER_CELL + UNITS_PER_CELL / 2).unwrap_or(u64::MAX) + CENTER_TOLERANCE;
    let later = u64::try_from(UNITS_PER_CELL).unwrap_or(u64::MAX) + CENTER_TOLERANCE;
    let distance = first.saturating_add(later.saturating_mul(steps.saturating_sub(1) as u64));
    let pulses = distance.div_ceil(per_pulse);
    pulses.saturating_mul(u64::from(MAX_HOLD_TICKS) + PER_PULSE_IDLE_TICKS) < fuse
}

fn movement_plan(
    actor: &crate::cpu::VisibleActor,
    current: CellId,
    next: CellId,
    memory: UtilityMemory,
) -> ControlPlan {
    let desired = direction(current, next).unwrap_or(actor.heading);
    ControlPlan {
        intent: Intent::moving(input_direction(actor, desired)),
        hold_ticks: hold_ticks(actor, next, desired),
        next_memory: memory,
    }
}

fn adjacent_block(observation: &Observation, cell: CellId) -> bool {
    STEPS
        .iter()
        .filter_map(|(_, dc, dr)| offset(cell, *dc, *dr))
        .any(|next| observation.parts().arena.blocks.contains(&next))
}
