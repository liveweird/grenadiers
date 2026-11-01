use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PulseRequest, RouteMode, certify_lair_contact_pulse, certify_lair_pulse,
};
use crate::cpu::{CpuRng, Intent, MobContactLease, MovementIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::compatibility::actor;
use super::steering::RefugeGuidance;
use super::{ExecutionCertificate, MAX_PLAN_AGE_TICKS, PlantEscapeOrigin, SurvivalPlan};

#[derive(Clone, Debug)]
pub struct LairSurvivalPlan {
    pub survival: SurvivalPlan,
    pub contact: Option<MobContactLease>,
}

#[path = "lair_entry.rs"]
mod entry;
pub use entry::{plan_lairs, plan_lairs_with_origin};
pub(super) use entry::{plan_lairs_v23, plan_lairs_v24, plan_lairs_v25};

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
    strategic_goals: bool,
    failed_plant_recovery: bool,
    persistent_burning_oil: bool,
    hard_tactics: bool,
    counter_pressure: bool,
    pickup_progress: bool,
    refuge_guidance: RefugeGuidance,
) -> Result<LairSurvivalPlan, PolicyError> {
    let parts = observation.parts();
    if parts.rules.version != rules_version
        || (rules_version == 8 && observation.lairs().is_none())
        || (matches!(rules_version, 9 | 10) && observation.respawn_lairs().is_none())
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
            control: ControlPlan::stop(if strategic_goals {
                UtilityMemory::new(memory.key()).carry_progress(*memory)
            } else {
                UtilityMemory::new(memory.key())
            }),
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
        let mut plan = super::origin_planner::plan_lairs(
            observation,
            history,
            settled_stop_tick,
            memory,
            origin,
            work,
        )
        .unwrap_or_else(stop);
        if strategic_goals {
            plan.survival.control.next_memory =
                plan.survival.control.next_memory.carry_progress(*memory);
        }
        return Ok(plan);
    }

    let ordinary = if rules_version == 8 {
        crate::cpu::policy::plan_lairs(observation, memory, rng, work)?
    } else {
        super::ordinary_planner::plan(
            observation,
            memory,
            rng,
            work,
            true,
            opportunistic_planting,
            strategic_goals,
            failed_plant_recovery,
            hard_tactics,
        )?
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
    let recovering_plant = failed_plant_recovery
        && super::strategic_recovery::attained_firing_position(observation, &ordinary);
    let plant_support =
        super::plant_tactic::PlantSupport::for_policy(persistent_burning_oil, counter_pressure);
    if opportunistic_planting
        && !recovering_plant
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
            strategic_goals,
            pickup_progress,
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
            pickup_progress,
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
    let target = super::steering::target(observation, &hazards, &ordinary, refuge_guidance, work)
        .or_else(|| actor.position.cell());
    let choices = super::steering::movement_choices(ordinary.intent.movement);

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
            let candidate = LairSurvivalPlan {
                survival: SurvivalPlan {
                    control: ControlPlan {
                        intent: Intent {
                            movement,
                            manual: None,
                        },
                        hold_ticks,
                        next_memory: if (opportunistic_planting
                            && ordinary.intent.movement == MovementIntent::Stop)
                            || (ordinary.intent.manual.is_none()
                                && movement == ordinary.intent.movement)
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
            let progresses = if recovering_plant {
                super::strategic_recovery::pulse_progresses(&candidate.survival, actor.position)
            } else {
                super::steering::certified_pulse_progresses(&candidate.survival, actor.position)
            };
            if best.as_ref().is_none_or(|(old_score, old_progresses, _)| {
                if recovering_plant && progresses != *old_progresses {
                    progresses
                } else {
                    score < *old_score || (score == *old_score && progresses && !*old_progresses)
                }
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
            strategic_goals,
            plant_support,
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
            plant_support,
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
    let mut fallback = fallback;
    if recovering_plant
        && super::strategic_recovery::pulse_progresses(&fallback.survival, actor.position)
    {
        fallback.survival.control.next_memory = fallback
            .survival
            .control
            .next_memory
            .defer_strategic_target(observation.parts().tick)
            .without_execution_authority();
    }
    Ok(fallback)
}
