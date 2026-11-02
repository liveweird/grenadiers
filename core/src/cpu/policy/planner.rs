use crate::cpu::{
    BombKnowledge, CpuRng, Intent, ManualIntent, MemoryKey, Observation, ObservedActor, WorkBudget,
};
use crate::game::CellId;

use super::execution::follow_committed_escape;
use super::goals::{select, select_v23, transit_allowed};
use super::hazards::{immediately_safe, planting_supported, unsupported_control_effect};
use super::movement::{bounded_alignment_hold_ticks, direction, hold_ticks, input_direction};
pub(super) use super::placement::alignment_direction;
use super::placement::{
    TurnAlignment, adjacent_block, centered_for_placement, ordinary_turn_alignment,
    placement_center_threshold,
};
use super::{ControlPlan, PolicyError, UtilityMemory};

#[path = "planner_escape.rs"]
mod escape;
use escape::{escape_before_fuse, escape_route};

#[path = "planner_guidance.rs"]
mod guidance;

#[path = "planner_version.rs"]
pub(super) mod version;
use version::PlannerVersion;

pub fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version >= 8 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::Legacy)
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
    plan_inner(observation, memory, rng, work, PlannerVersion::Lairs)
}

pub(crate) fn plan_respawn(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::Respawn)
}

pub(crate) fn plan_respawn_v23(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::RespawnV23)
}

pub(crate) fn plan_respawn_v24(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::RespawnV24)
}

pub(crate) fn plan_respawn_v25(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 9 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::RespawnV25)
}

pub(crate) fn plan_spike_pits_v26(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 10 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::SpikePitsV26)
}

pub(crate) fn plan_hard_v28(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 10 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::HardV28)
}

pub(crate) fn plan_farm_v38(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 10 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::FarmV38)
}

pub(crate) fn plan_hard_farm_v39(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<ControlPlan, PolicyError> {
    if observation.parts().rules.version != 10 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(observation, memory, rng, work, PlannerVersion::HardFarmV39)
}

pub(super) fn plan_inner(
    observation: &Observation,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    version: PlannerVersion,
) -> Result<ControlPlan, PolicyError> {
    let follow_committed = version.follow_committed();
    let persistent_goals = version.persistent_goals();
    let avoid_pits = version.avoid_pits();
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
        && (!avoid_pits || super::pit_avoidance::retained_step_allowed(observation, next, work))
    {
        return Ok(movement_plan(
            actor,
            current,
            next,
            advanced,
            parts.rules.settings.turn_assist_units() as u32,
        ));
    }

    // V23 planting is admitted only by the survival layer's full absent,
    // present, route, and contact proofs. Its ordinary planner supplies the
    // persistent firing objective but never emits an uncertified plant.
    let can_prepare_bomb = !persistent_goals
        && adjacent_block(observation, current)
        && planting_supported(observation);
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
        && let Some(route) = escape_route(observation, current, work)
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

    navigate(
        observation,
        actor,
        current,
        memory,
        rng,
        work,
        persistent_goals,
        version.strategic_goals(),
        version.failed_plant_recovery(),
        super::GoalModes {
            pressure: version.pressure_mode(),
            farm_origin: version.farm_origin_mode(),
        },
        version.farm_origin_recovery(),
        avoid_pits,
    )
}

#[allow(clippy::too_many_arguments)]
fn navigate(
    observation: &Observation,
    actor: &crate::cpu::VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    persistent_goals: bool,
    strategic_goals: bool,
    failed_plant_recovery: bool,
    goal_modes: super::GoalModes,
    farm_origin_recovery: bool,
    avoid_pits: bool,
) -> Result<ControlPlan, PolicyError> {
    let key = memory.key();
    if strategic_goals {
        let guidance = guidance::strategic(
            observation,
            actor,
            current,
            memory,
            work,
            goal_modes,
            failed_plant_recovery,
            farm_origin_recovery,
            avoid_pits,
        );
        let Some(guidance) = guidance else {
            return Ok(ControlPlan::stop(*memory));
        };
        let next_memory = UtilityMemory::from_route(key, &[guidance.first_step], false)
            .carry_progress(*memory)
            .with_strategic_intent(guidance.intent)
            .with_failed_target(guidance.failed_target);
        if guidance.first_step == current || !immediately_safe(observation, guidance.first_step) {
            return Ok(ControlPlan::stop(
                if immediately_safe(observation, current) {
                    next_memory
                } else if guidance.intent.objective == super::StrategicObjectiveKind::Wait {
                    next_memory.use_survival_fallback(observation.parts().tick)
                } else {
                    next_memory.mark_strategic_phase(
                        super::StrategicPhase::Evade,
                        super::StrategicSwitchReason::SafetyInterrupt,
                        observation.parts().tick,
                    )
                },
            ));
        }
        direction(current, guidance.first_step).ok_or(PolicyError::InvalidObservation)?;
        return Ok(movement_plan(
            actor,
            current,
            guidance.first_step,
            next_memory,
            observation.parts().rules.settings.turn_assist_units() as u32,
        ));
    }
    let guidance = if persistent_goals {
        select_v23(observation, actor, current, memory, work)
    } else {
        select(observation, actor, current, rng, work)
    };
    let Some(guidance) = guidance else {
        return Ok(ControlPlan::stop(if persistent_goals {
            *memory
        } else {
            UtilityMemory::new(key)
        }));
    };
    let next = guidance.first_step;
    if !immediately_safe(observation, next) {
        return Ok(ControlPlan::stop(if persistent_goals {
            *memory
        } else {
            UtilityMemory::from_route(key, &[next], false)
        }));
    }
    let next_memory = guidance.goal.map_or_else(
        || {
            let fresh = UtilityMemory::from_route(key, &[next], false);
            if persistent_goals {
                fresh.carry_progress(*memory).without_guidance(current)
            } else {
                fresh
            }
        },
        |goal| {
            UtilityMemory::from_route(key, &[next], false)
                .carry_progress(*memory)
                .with_progress(current, goal)
        },
    );
    if next == current {
        return Ok(ControlPlan::stop(next_memory));
    }
    direction(current, next).ok_or(PolicyError::InvalidObservation)?;
    Ok(movement_plan(
        actor,
        current,
        next,
        next_memory,
        observation.parts().rules.settings.turn_assist_units() as u32,
    ))
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
