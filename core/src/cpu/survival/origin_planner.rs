use crate::cpu::forecast::{
    ForecastLimits, HazardForecast, ObservedHazardMemory, TickWindow, forecast,
};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{
    PulseOriginGrace, PulseRequest, RouteMode, certify_escape_pulse, certify_lair_escape_pulse,
};
use crate::cpu::{Intent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::{LairSurvivalPlan, MAX_PLAN_AGE_TICKS, PlantEscapeOrigin, SurvivalPlan};

struct OriginPlan {
    survival: SurvivalPlan,
    contact: Option<crate::cpu::MobContactLease>,
}

pub(super) fn plan(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    plan_inner(
        observation,
        history,
        settled_stop_tick,
        memory,
        origin,
        false,
        work,
    )
    .map(|plan| plan.survival)
}

pub(super) fn plan_lairs(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let plan = plan_inner(
        observation,
        history,
        settled_stop_tick,
        memory,
        origin,
        true,
        work,
    )?;
    Some(LairSurvivalPlan {
        survival: plan.survival,
        contact: plan.contact,
    })
}

#[allow(clippy::too_many_arguments)]
fn plan_inner(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    lairs: bool,
    work: &mut WorkBudget,
) -> Option<OriginPlan> {
    origin.retain_stopped(observation)?;
    let forecast = forecast(observation, history, None, ForecastLimits::default(), work).ok()?;
    plan_with_forecast(
        observation,
        &forecast,
        settled_stop_tick,
        memory,
        origin,
        lairs,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
fn plan_with_forecast(
    observation: &Observation,
    forecast: &HazardForecast,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    lairs: bool,
    work: &mut WorkBudget,
) -> Option<OriginPlan> {
    let actor = super::compatibility::actor(observation)?;
    let target = super::steering::origin_target(observation, forecast, origin.cell(), work)?;
    let current_distance = super::steering::distance(actor.position, target);
    let basis = observation.parts().tick.0;
    let start_window = TickWindow {
        start: Tick(basis.checked_add(1)?),
        end: Tick(basis.checked_add(MAX_PLAN_AGE_TICKS + 2)?),
    };
    let mut best: Option<(u32, OriginPlan)> = None;
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        let hold_ticks = if mode == RouteMode::CertifiedOnly {
            12
        } else {
            3
        };
        let until = start_window.end.0.checked_add(u64::from(hold_ticks) + 9)?;
        for direction in [
            Direction::North,
            Direction::East,
            Direction::South,
            Direction::West,
        ] {
            let movement = MovementIntent::Direction(direction);
            let request = PulseRequest {
                movement,
                hold_ticks,
                start_window,
                stopped_safe_until: Tick(until),
                mode,
                settled_stop_tick,
            };
            let grace = PulseOriginGrace {
                cell: origin.cell(),
                bomb_id: origin.bomb_id(),
            };
            let (lease, contact) = if lairs {
                let Ok((lease, contact)) =
                    certify_lair_escape_pulse(observation, forecast, request, grace, work)
                else {
                    continue;
                };
                (lease, Some(contact))
            } else {
                let Ok(lease) = certify_escape_pulse(observation, forecast, request, grace, work)
                else {
                    continue;
                };
                (lease, None)
            };
            let score = super::steering::score(&lease, target);
            if score >= current_distance {
                continue;
            }
            let candidate = OriginPlan {
                survival: SurvivalPlan {
                    control: ControlPlan {
                        intent: Intent {
                            movement,
                            manual: None,
                        },
                        hold_ticks,
                        next_memory: UtilityMemory::new(memory.key()),
                    },
                    lease: Some(lease),
                    execution: None,
                },
                contact,
            };
            if best.as_ref().is_none_or(|(previous, _)| score < *previous) {
                best = Some((score, candidate));
            }
        }
        if best.is_some() {
            break;
        }
    }
    best.map(|(_, plan)| plan)
}

pub(super) fn plan_lairs_with_forecast(
    observation: &Observation,
    forecast: &HazardForecast,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let plan = plan_with_forecast(
        observation,
        forecast,
        settled_stop_tick,
        memory,
        origin,
        true,
        work,
    )?;
    Some(LairSurvivalPlan {
        survival: plan.survival,
        contact: plan.contact,
    })
}

pub(super) fn plan_disabled_with_forecast(
    observation: &Observation,
    forecast: &HazardForecast,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let plan = plan_with_forecast(
        observation,
        forecast,
        settled_stop_tick,
        memory,
        origin,
        false,
        work,
    )?;
    Some(LairSurvivalPlan {
        survival: plan.survival,
        contact: None,
    })
}
