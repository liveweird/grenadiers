use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PulseRequest, RouteMode, certify_lair_contact_pulse, certify_lair_pulse,
};
use crate::cpu::{CpuRng, Intent, MobContactLease, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::compatibility::actor;
use super::{ExecutionCertificate, MAX_PLAN_AGE_TICKS, PlantEscapeOrigin, SurvivalPlan};

#[derive(Clone, Debug)]
pub struct LairSurvivalPlan {
    pub survival: SurvivalPlan,
    pub contact: Option<MobContactLease>,
}

/// Rules-v8 policy. It certifies navigation and stopped handoff before trying
/// any optional tactic, so a failed tactic returns useful movement.
pub fn plan_lairs(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_with_origin(
        observation,
        memory,
        history,
        settled_stop_tick,
        None,
        rng,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_lairs_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        8,
        false,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_lairs_v23(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan_lairs_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        9,
        true,
        true,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_lairs_inner(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    rules_version: u16,
    pickup_handoff: bool,
    opportunistic_planting: bool,
) -> Result<LairSurvivalPlan, PolicyError> {
    let parts = observation.parts();
    if parts.rules.version != rules_version
        || (rules_version == 8 && observation.lairs().is_none())
        || (rules_version == 9 && observation.respawn_lairs().is_none())
    {
        return Err(PolicyError::InvalidObservation);
    }
    if memory.key().world != parts.world
        || memory.key().cpu_seat != parts.recipient
        || history.key() != memory.key()
        || rng.key() != memory.key()
    {
        return Err(PolicyError::IdentityMismatch);
    }
    let stop = || LairSurvivalPlan {
        survival: SurvivalPlan {
            control: ControlPlan::stop(UtilityMemory::new(memory.key())),
            lease: None,
            execution: None,
        },
        contact: None,
    };
    if settled_stop_tick > parts.tick {
        return Ok(stop());
    }
    let Some(actor) = actor(observation) else {
        return Err(PolicyError::InvalidObservation);
    };
    if !actor.alive || actor.voluntary_actions_frozen {
        return Ok(stop());
    }
    if let Some(origin) = origin {
        return Ok(super::origin_planner::plan_lairs(
            observation,
            history,
            settled_stop_tick,
            memory,
            origin,
            work,
        )
        .unwrap_or_else(stop));
    }

    let ordinary = if opportunistic_planting {
        crate::cpu::policy::plan_respawn_v23(observation, memory, rng, work)?
    } else if rules_version == 8 {
        crate::cpu::policy::plan_lairs(observation, memory, rng, work)?
    } else {
        crate::cpu::policy::plan_respawn(observation, memory, rng, work)?
    };
    let Ok(hazards) = forecast(observation, history, None, ForecastLimits::default(), work) else {
        return Ok(stop());
    };
    let Some(end) = parts.tick.0.checked_add(MAX_PLAN_AGE_TICKS + 2) else {
        return Ok(stop());
    };
    let Some(first) = parts.tick.0.checked_add(1) else {
        return Ok(stop());
    };
    let start_window = TickWindow {
        start: Tick(first),
        end: Tick(end),
    };
    // Keep the ordinary pulse when its complete swept path avoids every item.
    // Item collection can change movement physics, so a path that can collect
    // gets only one update before a fresh observation is required.
    const PICKUP_HOLD_TICKS: u8 = 1;
    let ordinary_hold = ordinary.hold_ticks.clamp(1, 12);
    let ordinary_clear = super::deadline::ordinary_movement_is_clear(
        observation,
        ordinary.intent.movement != MovementIntent::Stop,
        work,
    );
    if opportunistic_planting
        && let Some(plan) = super::lair_v23::opening(
            observation,
            &hazards,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            ordinary_clear,
            start_window,
            end,
            work,
        )
    {
        return Ok(plan);
    }
    if !opportunistic_planting
        && ordinary.intent.manual.is_none()
        && ordinary_clear
        && !parts.pickups.is_empty()
        && let Some(stopped_safe_until) = end.checked_add(u64::from(ordinary_hold) + 9).map(Tick)
        && let Ok((lease, contact)) = certify_lair_pulse(
            observation,
            &hazards,
            PulseRequest {
                movement: ordinary.intent.movement,
                hold_ticks: ordinary_hold,
                start_window,
                stopped_safe_until,
                mode: RouteMode::CertifiedOnly,
                settled_stop_tick,
            },
            work,
        )
    {
        return Ok(LairSurvivalPlan {
            survival: SurvivalPlan {
                control: ControlPlan {
                    intent: ordinary.intent,
                    hold_ticks: ordinary_hold,
                    next_memory: ordinary.next_memory,
                },
                lease: Some(lease),
                execution: None,
            },
            contact: Some(contact),
        });
    }
    if pickup_handoff
        && !opportunistic_planting
        && let Some(plan) = super::pickup_handoff::lair(
            observation,
            &hazards,
            ordinary,
            ordinary_clear,
            start_window,
            end,
            settled_stop_tick,
            work,
        )
    {
        return Ok(plan);
    }
    if !pickup_handoff
        && ordinary.intent.manual.is_none()
        && ordinary_clear
        && !parts.pickups.is_empty()
        && let Some(stopped_safe_until) =
            end.checked_add(u64::from(PICKUP_HOLD_TICKS) + 9).map(Tick)
        && let Ok(contact) = certify_lair_contact_pulse(
            observation,
            &hazards,
            PulseRequest {
                movement: ordinary.intent.movement,
                hold_ticks: PICKUP_HOLD_TICKS,
                start_window,
                stopped_safe_until,
                mode: RouteMode::CertifiedOnly,
                settled_stop_tick,
            },
            work,
        )
    {
        return Ok(LairSurvivalPlan {
            survival: SurvivalPlan {
                control: ControlPlan {
                    intent: ordinary.intent,
                    hold_ticks: PICKUP_HOLD_TICKS,
                    next_memory: ordinary.next_memory,
                },
                lease: None,
                execution: None,
            },
            contact: Some(contact),
        });
    }
    let target = super::steering::target(observation, &hazards, &ordinary, work)
        .or_else(|| actor.position.cell());
    let mut choices = Vec::with_capacity(5);
    choices.push(ordinary.intent.movement);
    for direction in [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ] {
        let movement = MovementIntent::Direction(direction);
        if !choices.contains(&movement) {
            choices.push(movement);
        }
    }
    if !choices.contains(&MovementIntent::Stop) {
        choices.push(MovementIntent::Stop);
    }

    let mut fallback = None;
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let mut best: Option<(u32, bool, LairSurvivalPlan)> = None;
        for &movement in &choices {
            let hold_ticks = match (movement, mode) {
                (MovementIntent::Stop, _) => 1,
                (_, RouteMode::AllowStationaryAssumptions) => 3,
                _ if movement == ordinary.intent.movement => ordinary.hold_ticks.clamp(1, 12),
                _ => 12,
            };
            let Some(until) = end.checked_add(u64::from(hold_ticks) + 9) else {
                return Ok(stop());
            };
            let request = PulseRequest {
                movement,
                hold_ticks,
                start_window,
                stopped_safe_until: Tick(until),
                mode,
                settled_stop_tick,
            };
            let Ok((lease, contact)) = certify_lair_pulse(observation, &hazards, request, work)
            else {
                continue;
            };
            let score = target.map_or(0, |target| super::steering::score(&lease, target));
            let end_pose = lease.pose_after(lease.hold_ticks());
            let progresses = movement != MovementIntent::Stop
                && (end_pose.min() != actor.position || end_pose.max() != actor.position);
            let candidate = LairSurvivalPlan {
                survival: SurvivalPlan {
                    control: ControlPlan {
                        intent: Intent {
                            movement,
                            manual: None,
                        },
                        hold_ticks,
                        next_memory: if opportunistic_planting
                            && ordinary.intent.movement == MovementIntent::Stop
                        {
                            ordinary.next_memory
                        } else if ordinary.intent.manual.is_none()
                            && movement == ordinary.intent.movement
                        {
                            ordinary.next_memory
                        } else if opportunistic_planting {
                            UtilityMemory::new(memory.key()).carry_progress(ordinary.next_memory)
                        } else {
                            UtilityMemory::new(memory.key())
                        },
                    },
                    lease: Some(lease),
                    execution: None,
                },
                contact: Some(contact),
            };
            if best.as_ref().is_none_or(|(old_score, old_progresses, _)| {
                score < *old_score || (score == *old_score && progresses && !*old_progresses)
            }) {
                best = Some((score, progresses, candidate));
            }
        }
        if let Some((_, _, plan)) = best {
            fallback = Some(plan);
            break;
        }
    }
    let Some(fallback) = fallback else {
        return Ok(stop());
    };

    let plant = if opportunistic_planting {
        super::plant_tactic::certified_plant_candidates(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            work,
        )
    } else {
        super::plant_tactic::certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            true,
            work,
        )
    };
    if let Some(plant) = plant
        && let Some(ExecutionCertificate::Plant(certificate)) = &plant.execution
        && let Ok(contact) = MobContactLease::certify_route(observation, certificate.route(), work)
    {
        return Ok(LairSurvivalPlan {
            survival: plant,
            contact: Some(contact),
        });
    }
    Ok(fallback)
}
