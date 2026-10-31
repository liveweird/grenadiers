use crate::cpu::forecast::{HazardForecast, ObservedHazardMemory, TickWindow};
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{PulseRequest, RouteMode, certify_lair_pulse};
use crate::cpu::{MobContactLease, Observation, WorkBudget};
use crate::game::Tick;

use super::{ExecutionCertificate, LairSurvivalPlan, SurvivalPlan};

#[allow(clippy::too_many_arguments)]
pub(super) fn opening(
    observation: &Observation,
    hazards: &HazardForecast,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    ordinary_clear: bool,
    start_window: TickWindow,
    end: u64,
    strategic: bool,
    work: &mut WorkBudget,
) -> Option<LairSurvivalPlan> {
    if ordinary.intent.manual.is_some() {
        return None;
    }
    let hold_ticks = ordinary.hold_ticks.clamp(1, 12);
    let stopped_safe_until = end.checked_add(u64::from(hold_ticks) + 9).map(Tick)?;
    let certified = certify_lair_pulse(
        observation,
        hazards,
        PulseRequest {
            movement: ordinary.intent.movement,
            hold_ticks,
            start_window,
            stopped_safe_until,
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick,
        },
        work,
    );
    let fallback = if let Ok((lease, contact)) = certified {
        LairSurvivalPlan {
            survival: SurvivalPlan {
                control: ControlPlan {
                    intent: ordinary.intent,
                    hold_ticks,
                    next_memory: ordinary.next_memory,
                },
                lease: Some(lease),
                execution: None,
            },
            contact: Some(contact),
        }
    } else {
        super::pickup_handoff::lair(
            observation,
            hazards,
            *ordinary,
            ordinary_clear,
            start_window,
            end,
            settled_stop_tick,
            work,
        )?
    };
    let mut tactic_work = WorkBudget::new(work.remaining().saturating_mul(7) / 8);
    let plant = super::plant_tactic::certified_plant_candidates(
        observation,
        history,
        settled_stop_tick,
        memory,
        ordinary,
        strategic,
        &mut tactic_work,
    );
    work.try_take(tactic_work.spent()).ok()?;
    let Some(plant) = plant else {
        return Some(fallback);
    };
    let Some(ExecutionCertificate::Plant(certificate)) = &plant.execution else {
        return Some(fallback);
    };
    let Ok(contact) = MobContactLease::certify_route(observation, certificate.route(), work) else {
        return Some(fallback);
    };
    Some(LairSurvivalPlan {
        survival: plant,
        contact: Some(contact),
    })
}
