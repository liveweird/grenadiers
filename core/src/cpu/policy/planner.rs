use std::collections::VecDeque;

use crate::cpu::{
    BombKnowledge, CpuRng, Intent, ManualIntent, MemoryKey, Observation, ObservedActor, WorkBudget,
};
use crate::game::CellId;
use crate::game::types::UNITS_PER_CELL;

use super::execution::follow_committed_escape;
use super::goals::{select, transit_allowed};
use super::hazards::{immediately_safe, planting_supported, unsupported_control_effect};
use super::movement::{
    STEPS, bounded_alignment_hold_ticks, direction, hold_ticks, input_direction, offset,
    pulse_minimum_distance,
};
pub(super) use super::placement::alignment_direction;
use super::placement::{
    TurnAlignment, centered_for_placement, ordinary_turn_alignment, placement_center_threshold,
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
    if observation.parts().rules.version == 8 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, true)
}

pub(crate) fn plan_lairs(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 8 || observation.lairs().is_none() {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, false)
}

fn plan_inner(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    follow_committed: bool,
) -> Result<ControlPlan, PolicyError> {
    let parts = observation.parts();
    let key = MemoryKey {
        world: parts.world,
        cpu_seat: parts.recipient,
    };
    if memory.key() != key || rng.key() != key {
        return Err(PolicyError::IdentityMismatch);
    }
    if follow_committed && let Some(committed) = follow_committed_escape(observation, memory)? {
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
        && transit_allowed(observation, actor, next)
    {
        return Ok(movement_plan(
            actor,
            current,
            next,
            advanced,
            parts.rules.settings.turn_assist_units() as u32,
        ));
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

fn navigate(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    current: CellId,
    key: MemoryKey,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    let Some(guidance) = select(observation, actor, current, rng, work) else {
        return Ok(ControlPlan::stop(UtilityMemory::new(key)));
    };
    let next = guidance.first_step;
    if !immediately_safe(observation, next) {
        return Ok(ControlPlan::stop(UtilityMemory::from_route(
            key,
            &[next],
            false,
        )));
    }
    direction(current, next).ok_or(PolicyError::InvalidObservation)?;
    Ok(movement_plan(
        actor,
        current,
        next,
        UtilityMemory::from_route(key, &[next], false),
        observation.parts().rules.settings.turn_assist_units() as u32,
    ))
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
    turn_tolerance: u32,
) -> ControlPlan {
    let desired = direction(current, next).unwrap_or(actor.heading);
    if let TurnAlignment::Correct(correction) =
        ordinary_turn_alignment(actor, current, desired, turn_tolerance)
    {
        return match bounded_alignment_hold_ticks(actor, current, correction, turn_tolerance) {
            Some(hold_ticks) => ControlPlan {
                intent: Intent::moving(input_direction(actor, correction)),
                hold_ticks,
                next_memory: memory,
            },
            None => ControlPlan::stop(memory),
        };
    }
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
