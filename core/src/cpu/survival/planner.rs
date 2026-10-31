use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PulseLease, PulseRequest, PunchRequest, RouteLimits, RouteMode, RouteRequest, RouteStep,
    certify_pulse, certify_respawn_pulse, plan_punch, plan_respawn_route, plan_route,
};
use crate::cpu::{CpuRng, Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::compatibility::actor;
use super::{ExecutionCertificate, MAX_PLAN_AGE_TICKS, PlantEscapeOrigin};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurvivalPlan {
    pub control: ControlPlan,
    pub lease: Option<PulseLease>,
    pub execution: Option<ExecutionCertificate>,
}

/// Produces one action; pulse leases never authorize manual input.
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
        control: ControlPlan::stop(UtilityMemory::new(memory.key())),
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
        return Ok(super::origin_planner::plan(
            observation,
            history,
            settled_stop_tick,
            memory,
            origin,
            work,
        )
        .unwrap_or_else(stop));
    }
    let ordinary = if respawn {
        crate::cpu::policy::plan_respawn(observation, memory, rng, work)?
    } else {
        crate::cpu::policy::plan(observation, memory, rng, work)?
    };
    let storm_enabled = observation.environment().is_some_and(|environment| {
        environment.storm_cloud_config() == crate::game::StormCloudConfig::Single
    });
    let clear_movement = super::deadline::ordinary_movement_is_clear(
        observation,
        ordinary.intent.movement != MovementIntent::Stop,
        work,
    );
    let bounded_tactics = storm_enabled || respawn;
    let bounded_plant_attempted =
        bounded_tactics && (ordinary.intent.manual.is_some() || clear_movement);
    if (ordinary.intent.manual.is_some() || clear_movement)
        && let Some(plan) = super::plant_tactic::certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            bounded_tactics,
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
    // Preserve the existing clear-board goals and pickup collection. Hazard
    // leases deliberately withhold item-changing paths until item tactics are
    // modeled; they must not replace the already legal clear-board behavior.
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
    let Some(route_survive_until) = super::deadline::ordinary_route_end(
        observation,
        start_window,
        forecast.horizon_end(),
        work,
    ) else {
        return Ok(stop());
    };
    let Some(target) = super::steering::target(observation, &forecast, &ordinary, work) else {
        return Ok(stop());
    };
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
                        next_memory: if *movement == ordinary.intent.movement
                            && ordinary.intent.manual.is_none()
                        {
                            ordinary.next_memory
                        } else {
                            UtilityMemory::new(memory.key())
                        },
                    },
                    lease: Some(lease),
                    execution: None,
                };
                if best.as_ref().is_none_or(|(previous, _)| score < *previous) {
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
                next_memory: UtilityMemory::new(memory.key()),
            },
            lease: None,
            execution: Some(ExecutionCertificate::Route(Box::new(route))),
        });
        break;
    }
    if (!bounded_plant_attempted || !bounded_tactics)
        && let Some(plan) = super::plant_tactic::certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            bounded_tactics,
            work,
        )
    {
        return Ok(plan);
    }
    if (storm_enabled || respawn)
        && clear_movement
        && fallback.as_ref().is_some_and(|plan| {
            certified_pulse_progresses(plan, actor.position)
                && route_fallback
                    .as_ref()
                    .is_some_and(|route| route.control.intent.movement == MovementIntent::Stop)
        })
    {
        return Ok(fallback.expect("progressing fallback was checked"));
    }
    Ok(route_fallback.or(fallback).unwrap_or_else(stop))
}

fn certified_pulse_progresses(
    plan: &SurvivalPlan,
    initial_position: crate::game::Position,
) -> bool {
    let Some(lease) = &plan.lease else {
        return false;
    };
    if lease.movement() == MovementIntent::Stop {
        return false;
    }
    let end = lease.pose_after(lease.hold_ticks());
    end.min() != initial_position || end.max() != initial_position
}

fn certified_punch(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    // Keep at least half the remaining allowance for ordinary survival if this
    // optional tactic cannot prove a complete escape. All child work is charged.
    let mut tactic_work = WorkBudget::new(work.remaining() / 2);
    let certificate = plan_punch(
        observation,
        history,
        PunchRequest {
            execution_tick,
            settled_stop_tick,
            limits: RouteLimits::default(),
        },
        &mut tactic_work,
    );
    work.try_take(tactic_work.spent()).ok()?;
    let certificate = certificate.ok()?;
    Some(SurvivalPlan {
        control: ControlPlan {
            intent: Intent::STOP.with_manual(ManualIntent::Equipment),
            hold_ticks: 1,
            next_memory: UtilityMemory::new(memory.key()),
        },
        lease: None,
        execution: Some(ExecutionCertificate::Punch(Box::new(certificate))),
    })
}
