use crate::cpu::{BombKnowledge, Intent, MemoryKey, Observation, ObservedActor};

use super::hazards::{escape_step_clear, unsupported_control_effect};
use super::movement::{direction, hold_ticks, input_direction};
use super::{ControlPlan, PolicyError, UtilityMemory};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EscapeExecution {
    Continue(ControlPlan),
    Wait(ControlPlan),
    Completed,
    Invalidated,
}

/// Follows an already committed bounded escape without invoking policy search.
///
/// `Some` is an active escape action and never contains a manual action. `None`
/// means there is no committed escape, or its refuge is reached and the
/// recipient-visible bomb and flame layers are clear.
pub fn follow_committed_escape(
    observation: &Observation,
    memory: &UtilityMemory,
) -> Result<Option<ControlPlan>, PolicyError> {
    Ok(match follow_checked_escape(observation, memory)? {
        EscapeExecution::Continue(plan) | EscapeExecution::Wait(plan) => Some(plan),
        EscapeExecution::Completed => None,
        EscapeExecution::Invalidated => Some(ControlPlan::stop(*memory)),
    })
}

/// Distinguishes refuge waiting from an invalid route so a controller can replan.
pub fn follow_checked_escape(
    observation: &Observation,
    memory: &UtilityMemory,
) -> Result<EscapeExecution, PolicyError> {
    let parts = observation.parts();
    let key = MemoryKey {
        world: parts.world,
        cpu_seat: parts.recipient,
    };
    if memory.key() != key {
        return Err(PolicyError::IdentityMismatch);
    }
    if memory.planted_origin().is_none() {
        return Ok(EscapeExecution::Completed);
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
        return Ok(EscapeExecution::Invalidated);
    }
    let current = actor
        .position
        .cell()
        .ok_or(PolicyError::InvalidObservation)?;
    if let BombKnowledge::Visible(bombs) = &parts.bombs
        && bombs
            .iter()
            .any(|bomb| Some(bomb.cell) != memory.planted_origin() || bomb.owner != actor.id)
    {
        return Ok(EscapeExecution::Invalidated);
    }
    if let Some((next, advanced)) = memory.next(current, actor.position) {
        if !escape_step_clear(observation, next) {
            return Ok(EscapeExecution::Invalidated);
        }
        let Some(desired) = committed_direction(actor, current, next) else {
            return Ok(EscapeExecution::Invalidated);
        };
        return Ok(EscapeExecution::Continue(ControlPlan {
            intent: Intent::moving(input_direction(actor, desired)),
            hold_ticks: hold_ticks(actor, next, desired),
            next_memory: advanced,
        }));
    }
    if memory.refuge() != Some(current) || !escape_step_clear(observation, current) {
        return Ok(EscapeExecution::Invalidated);
    }
    if matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        && parts.flames.is_empty()
    {
        Ok(EscapeExecution::Completed)
    } else {
        Ok(EscapeExecution::Wait(ControlPlan::stop(*memory)))
    }
}

fn committed_direction(
    actor: &crate::cpu::VisibleActor,
    current: crate::game::CellId,
    next: crate::game::CellId,
) -> Option<crate::game::Direction> {
    let center = crate::game::Position::center(if current == next { current } else { next });
    if current == next {
        let valid = match actor.heading {
            crate::game::Direction::North => {
                actor.position.x.abs_diff(center.x) <= 64 && actor.position.y > center.y
            }
            crate::game::Direction::East => {
                actor.position.y.abs_diff(center.y) <= 64 && actor.position.x < center.x
            }
            crate::game::Direction::South => {
                actor.position.x.abs_diff(center.x) <= 64 && actor.position.y < center.y
            }
            crate::game::Direction::West => {
                actor.position.y.abs_diff(center.y) <= 64 && actor.position.x > center.x
            }
        };
        return valid.then_some(actor.heading);
    }
    let desired = direction(current, next)?;
    let within_corridor = match desired {
        crate::game::Direction::North | crate::game::Direction::South => {
            actor.position.x.abs_diff(center.x) <= 64
        }
        crate::game::Direction::East | crate::game::Direction::West => {
            actor.position.y.abs_diff(center.y) <= 64
        }
    };
    within_corridor.then_some(desired)
}
