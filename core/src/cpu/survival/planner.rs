use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{
    PlantCertificate, PlantRequest, PulseLease, PulseRequest, RouteCertificate, RouteLimits,
    RouteMode, RouteRequest, RouteStep, certify_pulse, plan_ordinary_plant, plan_route,
    plan_timed_weapon_plant,
};
use crate::cpu::{CpuRng, Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::MAX_PLAN_AGE_TICKS;
use super::compatibility::actor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurvivalPlan {
    pub control: ControlPlan,
    pub lease: Option<PulseLease>,
    pub execution: Option<ExecutionCertificate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCertificate {
    Route(Box<RouteCertificate>),
    Plant(Box<PlantCertificate>),
}

impl ExecutionCertificate {
    #[must_use]
    pub fn route(&self) -> &RouteCertificate {
        match self {
            Self::Route(route) => route,
            Self::Plant(plant) => plant.route(),
        }
    }

    #[must_use]
    pub const fn plant(&self) -> Option<&PlantCertificate> {
        match self {
            Self::Route(_) => None,
            Self::Plant(plant) => Some(plant),
        }
    }
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
    let ordinary = crate::cpu::policy::plan(observation, memory, rng, work)?;
    let clear_movement = super::deadline::ordinary_movement_is_clear(
        observation,
        ordinary.intent.movement != MovementIntent::Stop,
        work,
    );
    if (ordinary.intent.manual.is_some() || clear_movement)
        && let Some(plan) = certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            &ordinary,
            work,
        )
    {
        return Ok(plan);
    }
    if ordinary.intent.manual.is_some() {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
            execution: None,
        });
    }
    // Preserve the existing clear-board goals and pickup collection. Hazard
    // leases deliberately withhold item-changing paths until item tactics are
    // modeled; they must not replace the already legal clear-board behavior.
    if clear_movement {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
            execution: None,
        });
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
            if let Ok(lease) = certify_pulse(observation, &forecast, request, work) {
                let score = super::steering::score(&lease, target);
                let candidate = SurvivalPlan {
                    control: ControlPlan {
                        intent: Intent {
                            movement: *movement,
                            manual: None,
                        },
                        hold_ticks,
                        next_memory: if *movement == ordinary.intent.movement {
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
        let Ok(route) = plan_route(observation, &forecast, request, work) else {
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
    if let Some(plan) = certified_plant(
        observation,
        history,
        settled_stop_tick,
        memory,
        &ordinary,
        work,
    ) {
        return Ok(plan);
    }
    Ok(route_fallback.or(fallback).unwrap_or_else(stop))
}

fn certified_plant(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    if ordinary.intent.manual == Some(ManualIntent::Equipment) {
        return None;
    }
    if !super::plant_value::could_be_useful(observation, work) {
        return None;
    }
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    let request = PlantRequest {
        execution_tick,
        settled_stop_tick,
        first_movement: ordinary.intent.movement,
        limits: RouteLimits::default(),
    };
    let certificate = if actor(observation)?.weapon.is_some() {
        plan_timed_weapon_plant(observation, history, request, work)
    } else {
        plan_ordinary_plant(observation, history, request, work)
    }
    .ok()?;
    super::plant_value::useful_plant(observation, certificate.placement(), work)?.then(|| {
        SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement: ordinary.intent.movement,
                    manual: Some(ManualIntent::Bomb),
                },
                hold_ticks: 1,
                next_memory: UtilityMemory::new(memory.key()),
            },
            lease: None,
            execution: Some(ExecutionCertificate::Plant(Box::new(certificate))),
        }
    })
}
