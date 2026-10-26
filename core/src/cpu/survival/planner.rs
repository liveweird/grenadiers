use crate::cpu::forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast};
use crate::cpu::policy::{ControlPlan, PolicyError, UtilityMemory};
use crate::cpu::routing::{PulseLease, PulseRequest, RouteMode, certify_pulse};
use crate::cpu::{BombKnowledge, CpuRng, Intent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::MAX_PLAN_AGE_TICKS;
use super::compatibility::actor;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurvivalPlan {
    pub control: ControlPlan,
    pub lease: Option<PulseLease>,
}

/// Produces at most one executable action. Existing narrow planting retains its
/// independent escape proof; a pulse lease never authorizes a manual action.
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
    if ordinary.intent.manual.is_some() {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
        });
    }
    // Preserve the existing clear-board goals and pickup collection. Hazard
    // leases deliberately withhold item-changing paths until item tactics are
    // modeled; they must not replace the already legal clear-board behavior.
    if matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        && parts.flames.is_empty()
        && parts.oil.is_empty()
        && ordinary.intent.movement != MovementIntent::Stop
    {
        return Ok(SurvivalPlan {
            control: ordinary,
            lease: None,
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
    let Some(target) = super::steering::target(observation, &forecast, actor.position, work) else {
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
                };
                if best.as_ref().is_none_or(|(previous, _)| score < *previous) {
                    best = Some((score, candidate));
                }
            }
        }
        if let Some((_, candidate)) = best {
            return Ok(candidate);
        }
    }
    Ok(stop())
}
