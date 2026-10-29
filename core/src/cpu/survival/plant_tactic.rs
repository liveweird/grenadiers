use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{
    PlantRequest, RouteLimits, plan_ordinary_plant, plan_timed_weapon_plant,
};
use crate::cpu::{Intent, ManualIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::compatibility::actor;
use super::{ExecutionCertificate, SurvivalPlan};

pub(super) fn certified_plant(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    bounded: bool,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    if bounded {
        // Optional planting must leave enough of the shared decision allowance
        // for a certified movement fallback. Charge all attempted child work,
        // including a failed proof, back to the parent budget.
        let mut tactic_work = WorkBudget::new(work.remaining() / 2);
        let result = certified_plant_inner(
            observation,
            history,
            settled_stop_tick,
            memory,
            ordinary,
            &mut tactic_work,
        );
        work.try_take(tactic_work.spent()).ok()?;
        return result;
    }
    certified_plant_inner(
        observation,
        history,
        settled_stop_tick,
        memory,
        ordinary,
        work,
    )
}

fn certified_plant_inner(
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
