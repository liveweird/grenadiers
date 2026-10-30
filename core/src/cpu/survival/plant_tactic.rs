use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{
    PlantRequest, RouteLimits, plan_lair_ordinary_plant, plan_lair_timed_weapon_plant,
    plan_ordinary_plant, plan_timed_weapon_plant,
};
use crate::cpu::{BombKnowledge, Intent, ManualIntent, Observation, WorkBudget};
use crate::game::{OilBarrelState, PickupKind, Tick};

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
    if matches!(observation.parts().rules.version, 8 | 9)
        && !teleporter_plant_supported(observation)
    {
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
    let lairs = matches!(observation.parts().rules.version, 8 | 9);
    let certificate = if actor(observation)?.weapon.is_some() && lairs {
        plan_lair_timed_weapon_plant(observation, history, request, work)
    } else if actor(observation)?.weapon.is_some() {
        plan_timed_weapon_plant(observation, history, request, work)
    } else if lairs {
        plan_lair_ordinary_plant(observation, history, request, work)
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

fn teleporter_plant_supported(observation: &Observation) -> bool {
    let parts = observation.parts();
    !parts.rules.settings.enabled_items().contains_teleporter()
        || (parts
            .pickups
            .iter()
            .all(|pickup| pickup.kind != PickupKind::Teleporter)
            && parts.flames.is_empty()
            && matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
            && parts
                .oil
                .iter()
                .all(|oil| oil.state != OilBarrelState::Burning))
}
