use super::{
    ExecutionCertificate, MAX_PLAN_AGE_TICKS, PlantEscapeOrigin, SurvivalPlan,
    compatibility::actor, punch_tactic::certified_punch, steering::RefugeGuidance,
};
use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PulseRequest, RouteLimits, RouteMode, RouteRequest, RouteStep, certify_pulse,
    certify_respawn_pulse, plan_respawn_route, plan_route,
};
use crate::cpu::{CpuRng, Intent, MovementIntent, Observation, WorkBudget};
use crate::game::Tick;
pub fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<SurvivalPlan, PolicyError> {
    if observation.parts().rules.version >= 8 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_with_origin(
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
pub fn plan_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<SurvivalPlan, PolicyError> {
    if observation.parts().rules.version >= 8 {
        return Err(PolicyError::InvalidObservation);
    }
    plan_inner(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        false,
        RefugeGuidance::default(),
    )
}
#[allow(clippy::too_many_arguments)]
pub(super) fn plan_inner(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    respawn: bool,
    pickup_handoff: bool,
    opportunistic_planting: bool,
    strategic_goals: bool,
    failed_plant_recovery: bool,
    persistent_burning_oil: bool,
    hard_tactics: bool,
    counter_pressure: bool,
    pickup_progress: bool,
    refuge_guidance: RefugeGuidance,
) -> Result<SurvivalPlan, PolicyError> {
    let parts = observation.parts();
    if memory.key().world != parts.world
        || memory.key().cpu_seat != parts.recipient
        || history.key() != memory.key()
        || rng.key() != memory.key()
    {
        return Err(PolicyError::IdentityMismatch);
    }
    let stop = || SurvivalPlan {
        control: ControlPlan::stop(if strategic_goals {
            UtilityMemory::new(memory.key()).carry_progress(*memory)
        } else {
            UtilityMemory::new(memory.key())
        }),
        lease: None,
        execution: None,
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
        let mut plan = super::origin_planner::plan(
            observation,
            history,
            settled_stop_tick,
            memory,
            origin,
            work,
        )
        .unwrap_or_else(stop);
        if strategic_goals {
            plan.control.next_memory = plan.control.next_memory.carry_progress(*memory);
        }
        return Ok(plan);
    }
    let ordinary = super::ordinary_planner::plan(
        observation,
        memory,
        rng,
        work,
        respawn,
        opportunistic_planting,
        strategic_goals,
        failed_plant_recovery,
        hard_tactics,
        false,
    )?;
    let storm_enabled = observation.environment().is_some_and(|environment| {
        environment.storm_cloud_config() == crate::game::StormCloudConfig::Single
    });
    let clear_movement = super::deadline::ordinary_movement_is_clear(
        observation,
        ordinary.intent.movement != MovementIntent::Stop,
        work,
    );
    let recovering_plant = failed_plant_recovery
        && super::strategic_recovery::attained_firing_position(observation, &ordinary);
    let bounded_tactics = storm_enabled || respawn;
    let plant_support =
        super::plant_tactic::PlantSupport::for_policy(persistent_burning_oil, counter_pressure);
    let bounded_plant_attempted =
        bounded_tactics && (ordinary.intent.manual.is_some() || clear_movement || recovering_plant);
    if (ordinary.intent.manual.is_some() || clear_movement || recovering_plant)
        && let Some(plan) = super::plant_tactic::certified_plant_for_policy(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            bounded_tactics,
            opportunistic_planting,
            strategic_goals,
            plant_support,
            work,
        )
    {
        return Ok(plan);
    }
    if ordinary.intent.manual.is_some() && !storm_enabled && !respawn {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
            execution: None,
        });
    }
    if clear_movement && ordinary.intent.manual.is_none() && !respawn {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
            execution: None,
        });
    }
    if actor.equipment == Some(crate::game::Equipment::Punch)
        && let Some(plan) = certified_punch(observation, history, settled_stop_tick, memory, work)
    {
        return Ok(plan);
    }
    let Ok(forecast) = forecast(observation, history, None, ForecastLimits::default(), work) else {
        return Ok(stop());
    };
    let Some(end) = parts.tick.0.checked_add(MAX_PLAN_AGE_TICKS + 2) else {
        return Ok(stop());
    };
    let start_window = TickWindow {
        start: Tick(parts.tick.0 + 1),
        end: Tick(end),
    };
    if pickup_handoff
        && let Some(plan) = super::pickup_handoff::disabled(
            observation,
            &forecast,
            ordinary,
            clear_movement,
            start_window,
            end,
            settled_stop_tick,
            pickup_progress,
            work,
        )
    {
        return Ok(plan);
    }
    let Some(route_survive_until) = super::deadline::ordinary_route_end(
        observation,
        start_window,
        forecast.horizon_end(),
        work,
    ) else {
        return Ok(stop());
    };
    let Some(target) =
        super::steering::target(observation, &forecast, &ordinary, refuge_guidance, work)
    else {
        return Ok(stop());
    };
    let choices = super::steering::movement_choices(ordinary.intent.movement);
    let mut fallback = None;
    let mut route_fallback = None;
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let mut best: Option<(u32, SurvivalPlan)> = None;
        for movement in &choices {
            let hold_ticks = match (movement, mode) {
                (MovementIntent::Stop, _) => 1,
                (_, RouteMode::AllowStationaryAssumptions) => 3,
                _ if *movement == ordinary.intent.movement => ordinary.hold_ticks.clamp(1, 12),
                _ => 12,
            };
            let Some(until) = end.checked_add(u64::from(hold_ticks) + 9) else {
                return Ok(stop());
            };
            let request = PulseRequest {
                movement: *movement,
                hold_ticks,
                start_window,
                stopped_safe_until: Tick(until),
                mode,
                settled_stop_tick,
            };
            let certified = if respawn {
                certify_respawn_pulse(observation, &forecast, request, work)
            } else {
                certify_pulse(observation, &forecast, request, work)
            };
            if let Ok(lease) = certified {
                let score = super::steering::score(&lease, target);
                let candidate = SurvivalPlan {
                    control: ControlPlan {
                        intent: Intent {
                            movement: *movement,
                            manual: None,
                        },
                        hold_ticks,
                        next_memory: if (opportunistic_planting
                            && ordinary.intent.movement == MovementIntent::Stop)
                            || (ordinary.intent.manual.is_none()
                                && *movement == ordinary.intent.movement)
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
                };
                if best.as_ref().is_none_or(|(previous, old)| {
                    super::strategic_recovery::prefer_pulse(
                        recovering_plant,
                        &candidate,
                        score,
                        old,
                        *previous,
                        actor.position,
                    )
                }) {
                    best = Some((score, candidate));
                }
            }
        }
        if let Some((_, candidate)) = best {
            fallback = Some(candidate);
            break;
        }
    }
    if let Some(plan) = super::bracelet::plan(
        observation,
        &forecast,
        settled_stop_tick,
        memory,
        &ordinary,
        work,
    ) {
        return Ok(plan);
    }
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let request = RouteRequest {
            start_window,
            settled_stop_tick,
            survive_until: route_survive_until,
            mode,
            limits: RouteLimits::default(),
        };
        let routed = if respawn {
            plan_respawn_route(observation, &forecast, request, work)
        } else {
            plan_route(observation, &forecast, request, work)
        };
        let Ok(route) = routed else {
            continue;
        };
        let (movement, hold_ticks) = match route.steps().first() {
            Some(RouteStep::Move { direction, updates }) => {
                (MovementIntent::Direction(*direction), *updates)
            }
            Some(RouteStep::Wait { .. }) => (MovementIntent::Stop, 1),
            None => continue,
        };
        route_fallback = Some(SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement,
                    manual: None,
                },
                hold_ticks,
                next_memory: if opportunistic_planting {
                    UtilityMemory::new(memory.key()).carry_progress(ordinary.next_memory)
                } else {
                    UtilityMemory::new(memory.key())
                },
            },
            lease: None,
            execution: Some(ExecutionCertificate::Route(Box::new(route))),
        });
        break;
    }
    if (!bounded_plant_attempted || !bounded_tactics)
        && let Some(plan) = super::plant_tactic::certified_plant_for_policy(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            bounded_tactics,
            opportunistic_planting,
            strategic_goals,
            plant_support,
            work,
        )
    {
        return Ok(plan);
    }
    if (storm_enabled || respawn)
        && (clear_movement || observation.parts().rules.version == 10)
        && fallback.as_ref().is_some_and(|plan| {
            super::steering::certified_pulse_progresses(plan, actor.position)
                && route_fallback
                    .as_ref()
                    .is_some_and(|route| route.control.intent.movement == MovementIntent::Stop)
        })
    {
        return Ok(fallback.expect("progressing fallback was checked"));
    }
    if let Some(plan) = super::strategic_recovery::recover_progressing(
        recovering_plant,
        fallback.clone(),
        actor.position,
        observation.parts().tick,
    ) {
        return Ok(plan);
    }
    Ok(route_fallback.or(fallback).unwrap_or_else(stop))
}
