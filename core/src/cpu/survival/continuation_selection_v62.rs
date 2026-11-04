use crate::cpu::forecast::{ForecastLimits, HazardForecast, ObservedHazardMemory, forecast};
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::{ContinuationCertificate, continuation_candidates_v62};
use crate::cpu::{Observation, WorkBudget};
use crate::game::Tick;

use super::PlantEscapeOrigin;
use super::continuation_selection::{
    GENERATION_ALLOWANCE, OriginRecoveryChoice, continuation_recovery_memory, continuation_request,
    select_two_slots, v54_optional_allowance,
};

/// Versioned recovery selector whose only difference from v54 is accepting a
/// nonfinal-axis crossing before the unchanged complete continuation proof.
pub fn plan_lairs_v62(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<OriginRecoveryChoice> {
    let incoming = work.remaining();
    let common_start = work.spent();
    super::continuation_planner::charge_origin_validation(origin, observation, work).ok()?;
    origin.retain_stopped(observation)?;
    let forecast = forecast(observation, history, None, ForecastLimits::default(), work).ok()?;

    let common = work.spent().checked_sub(common_start)?;
    let optional_limit = v54_optional_allowance(incoming, common);
    let mut optional = WorkBudget::new(optional_limit);
    let continuation = try_continuation(
        observation,
        settled_stop_tick,
        origin,
        &forecast,
        &mut optional,
    );
    work.try_take(optional.spent()).ok()?;
    if let Some(certificate) = continuation {
        return Some(OriginRecoveryChoice::Continuation(Box::new(certificate)));
    }

    let fallback = if observation.respawn_lairs().is_some() {
        super::origin_planner::plan_lairs_with_forecast
    } else {
        super::origin_planner::plan_disabled_with_forecast
    };
    let mut plan = fallback(
        observation,
        &forecast,
        settled_stop_tick,
        memory,
        origin,
        true,
        work,
    )?;
    plan.survival.control.next_memory = continuation_recovery_memory(memory);
    Some(OriginRecoveryChoice::Pulse(Box::new(plan)))
}

fn try_continuation(
    observation: &Observation,
    settled_stop_tick: Tick,
    origin: &PlantEscapeOrigin,
    forecast: &HazardForecast,
    work: &mut WorkBudget,
) -> Option<ContinuationCertificate> {
    let request = continuation_request(observation, settled_stop_tick, forecast, work)?;
    let (refuge, first_axis) = origin.continuation_hint(work)?;
    let generation_limit = work.remaining().min(GENERATION_ALLOWANCE);
    let mut generation = WorkBudget::new(generation_limit);
    let generated = continuation_candidates_v62(
        observation,
        forecast,
        request,
        refuge,
        first_axis,
        &mut generation,
    );
    work.try_take(generation.spent()).ok()?;
    let candidates = generated.ok()?;
    let remaining = work.remaining();
    select_two_slots(&candidates, remaining, work, |candidate, proof| {
        super::continuation_planner::certify_continuation_v62(
            observation,
            forecast,
            request,
            candidate,
            origin,
            proof,
        )
        .ok()
    })
}
