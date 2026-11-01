//! Shared-budget selection between inactive continuation guidance and legacy recovery.

use crate::cpu::forecast::{ForecastLimits, HazardForecast, ObservedHazardMemory, forecast};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{
    ContinuationCertificate, ContinuationRequest, NominalLeg, continuation_candidates,
};
use crate::cpu::{BombKnowledge, Observation, WorkBudget};
use crate::game::Tick;

use super::{LairSurvivalPlan, MAX_PLAN_AGE_TICKS, PlantEscapeOrigin, SurvivalPlan};

const GENERATION_ALLOWANCE: u32 = 2_048;

pub enum OriginRecoveryChoice {
    Continuation(Box<ContinuationCertificate>),
    Pulse(Box<LairSurvivalPlan>),
}

#[must_use]
pub fn continuation_recovery_memory(previous: &UtilityMemory) -> UtilityMemory {
    UtilityMemory::new(previous.key()).carry_progress(*previous)
}

#[must_use]
pub fn continuation_recovery_stop(previous: &UtilityMemory) -> LairSurvivalPlan {
    LairSurvivalPlan {
        survival: SurvivalPlan {
            control: ControlPlan::stop(continuation_recovery_memory(previous)),
            lease: None,
            execution: None,
        },
        contact: None,
    }
}

pub fn plan_lairs(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<OriginRecoveryChoice> {
    let allowance = work.remaining() / 2;
    let mut attempt = WorkBudget::new(allowance);
    let prepared = prepare(
        observation,
        history,
        settled_stop_tick,
        origin,
        &mut attempt,
    );
    work.try_take(attempt.spent()).ok()?;
    let prepared = prepared?;
    if let Some(certificate) = prepared.continuation {
        return Some(OriginRecoveryChoice::Continuation(Box::new(certificate)));
    }
    let fallback = if observation.respawn_lairs().is_some() {
        super::origin_planner::plan_lairs_with_forecast
    } else {
        super::origin_planner::plan_disabled_with_forecast
    };
    let mut plan = fallback(
        observation,
        &prepared.forecast,
        settled_stop_tick,
        memory,
        origin,
        work,
    )?;
    plan.survival.control.next_memory = continuation_recovery_memory(memory);
    Some(OriginRecoveryChoice::Pulse(Box::new(plan)))
}

struct Prepared {
    forecast: HazardForecast,
    continuation: Option<ContinuationCertificate>,
}

fn prepare(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Option<Prepared> {
    super::continuation_planner::charge_origin_validation(origin, observation, work).ok()?;
    origin.retain_stopped(observation)?;
    let forecast = forecast(observation, history, None, ForecastLimits::default(), work).ok()?;
    let continuation = try_continuation(observation, settled_stop_tick, origin, &forecast, work);
    Some(Prepared {
        forecast,
        continuation,
    })
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
    let generated = continuation_candidates(
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
        super::continuation_planner::certify_continuation(
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

fn select_two_slots<T>(
    candidates: &[Vec<NominalLeg>],
    remaining: u32,
    work: &mut WorkBudget,
    mut prove: impl FnMut(&[NominalLeg], &mut WorkBudget) -> Option<T>,
) -> Option<T> {
    let slots = [remaining / 2, remaining - remaining / 2];
    for (candidate, slot) in candidates.iter().zip(slots) {
        let mut proof = WorkBudget::new(slot);
        let result = prove(candidate, &mut proof);
        work.try_take(proof.spent()).ok()?;
        if result.is_some() {
            return result;
        }
    }
    None
}

fn continuation_request(
    observation: &Observation,
    settled_stop_tick: Tick,
    forecast: &HazardForecast,
    work: &mut WorkBudget,
) -> Option<ContinuationRequest> {
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        return None;
    };
    work.try_take(u32::try_from(bombs.len()).ok()?.checked_add(1)?)
        .ok()?;
    let last_detonation = bombs
        .iter()
        .try_fold(observation.parts().tick, |last, bomb| {
            let crate::cpu::ObservedBombTrigger::Timed {
                remaining_ticks,
                paused: false,
            } = bomb.trigger
            else {
                return None;
            };
            Some(
                last.max(Tick(
                    observation
                        .parts()
                        .tick
                        .0
                        .checked_add(u64::from(remaining_ticks))?,
                )),
            )
        })?;
    let flame_ticks = crate::cpu::rules_config(observation)?.flame_ticks();
    let survive_until = Tick(last_detonation.0.checked_add(flame_ticks)?.checked_add(1)?);
    let first_update = Tick(
        observation
            .parts()
            .tick
            .0
            .checked_add(MAX_PLAN_AGE_TICKS + 1)?,
    );
    (survive_until <= forecast.horizon_end()).then_some(ContinuationRequest {
        first_update,
        settled_stop_tick,
        survive_until,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Direction;

    #[test]
    fn frozen_slots_preserve_the_second_after_first_exhaustion() {
        let candidates = vec![
            vec![NominalLeg::Move {
                direction: Direction::East,
                updates: 1,
            }],
            vec![NominalLeg::Move {
                direction: Direction::West,
                updates: 1,
            }],
        ];
        let mut work = WorkBudget::new(101);
        let mut calls = 0;
        let selected = select_two_slots(&candidates, 101, &mut work, |_, slot| {
            calls += 1;
            if calls == 1 {
                assert_eq!(slot.limit(), 50);
                slot.try_take(50).unwrap();
                None
            } else {
                assert_eq!(slot.limit(), 51);
                slot.try_take(7).unwrap();
                Some(9)
            }
        });
        assert_eq!(selected, Some(9));
        assert_eq!(work.spent(), 57);
    }
}
