use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::policy::ControlPlan;
use crate::cpu::routing::{PulseRequest, RouteMode, certify_pickup_pulse};
use crate::cpu::{Observation, WorkBudget};
use crate::game::Tick;

use super::{LairSurvivalPlan, SurvivalPlan};

#[allow(clippy::too_many_arguments)]
pub(super) fn lair(
    observation: &Observation,
    forecast: &HazardForecast,
    ordinary: ControlPlan,
    ordinary_clear: bool,
    start_window: TickWindow,
    end: u64,
    settled_stop_tick: Tick,
    pickup_progress: bool,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    let (survival, contact) = certify(
        observation,
        forecast,
        ordinary,
        ordinary_clear,
        start_window,
        end,
        settled_stop_tick,
        true,
        pickup_progress,
        work,
    )?;
    Some(LairSurvivalPlan { survival, contact })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn disabled(
    observation: &Observation,
    forecast: &HazardForecast,
    ordinary: ControlPlan,
    ordinary_clear: bool,
    start_window: TickWindow,
    end: u64,
    settled_stop_tick: Tick,
    pickup_progress: bool,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    certify(
        observation,
        forecast,
        ordinary,
        ordinary_clear,
        start_window,
        end,
        settled_stop_tick,
        false,
        pickup_progress,
        work,
    )
    .map(|(plan, _)| plan)
}

#[allow(clippy::too_many_arguments)]
fn certify(
    observation: &Observation,
    forecast: &HazardForecast,
    ordinary: ControlPlan,
    ordinary_clear: bool,
    start_window: TickWindow,
    end: u64,
    settled_stop_tick: Tick,
    with_contact: bool,
    pickup_progress: bool,
    work: &mut WorkBudget,
) -> Option<(SurvivalPlan, Option<crate::cpu::MobContactLease>)> {
    if ordinary.intent.manual.is_some()
        || (!ordinary_clear && !pickup_progress)
        || observation.parts().pickups.is_empty()
    {
        return None;
    }
    let stopped_safe_until = end.checked_add(10).map(Tick)?;
    let (lease, contact) = certify_pickup_pulse(
        observation,
        forecast,
        PulseRequest {
            movement: ordinary.intent.movement,
            hold_ticks: 1,
            start_window,
            stopped_safe_until,
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick,
        },
        with_contact,
        work,
    )
    .ok()?;
    Some((
        SurvivalPlan {
            control: ControlPlan {
                intent: ordinary.intent,
                hold_ticks: 1,
                next_memory: ordinary.next_memory,
            },
            lease: Some(lease),
            execution: None,
        },
        contact,
    ))
}
