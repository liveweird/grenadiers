use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{ControlPlan, UtilityMemory};
use crate::cpu::routing::{
    PlantRequest, RouteLimits, plan_lair_ordinary_plant, plan_lair_timed_weapon_plant,
    plan_ordinary_plant, plan_timed_weapon_plant,
};
use crate::cpu::{BombKnowledge, Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, OilBarrelState, PickupKind, Tick};

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

#[allow(clippy::too_many_arguments)]
pub(super) fn certified_plant_for_policy(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    bounded: bool,
    candidates: bool,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    if candidates {
        if !bounded {
            return certified_plant_candidates(
                observation,
                history,
                settled_stop_tick,
                memory,
                ordinary,
                work,
            );
        }
        let mut tactic_work = WorkBudget::new(work.remaining() / 2);
        let result = certified_plant_candidates(
            observation,
            history,
            settled_stop_tick,
            memory,
            ordinary,
            &mut tactic_work,
        );
        work.try_take(tactic_work.spent()).ok()?;
        result
    } else {
        certified_plant(
            observation,
            history,
            settled_stop_tick,
            memory,
            ordinary,
            bounded,
            work,
        )
    }
}

/// V23 optional tactic. A cheap shared-geometry check runs before any escape
/// proof, then every distinct first input receives a reserved bounded slice.
/// Failed proofs are charged and cannot consume the later-candidate reserve.
pub(super) fn certified_plant_candidates(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    let prefilter_allowance = work.remaining() / 8;
    let mut prefilter_work = WorkBudget::new(prefilter_allowance);
    let useful = super::plant_value::could_be_useful(observation, &mut prefilter_work);
    work.try_take(prefilter_work.spent()).ok()?;
    if !useful {
        return None;
    }

    let mut movements = Vec::with_capacity(5);
    movements.push(ordinary.intent.movement);
    for movement in [
        MovementIntent::Stop,
        MovementIntent::Direction(Direction::North),
        MovementIntent::Direction(Direction::East),
        MovementIntent::Direction(Direction::South),
        MovementIntent::Direction(Direction::West),
    ] {
        if movements.len() == 5 {
            break;
        }
        if !movements.contains(&movement) {
            movements.push(movement);
        }
    }
    let total = work.remaining();
    let count = u32::try_from(movements.len()).ok()?;
    let reserve = total / count.saturating_mul(2);
    for (index, movement) in movements.into_iter().enumerate() {
        let slice = if index == 0 {
            total.saturating_sub(reserve.saturating_mul(count.saturating_sub(1)))
        } else {
            reserve
        };
        let candidate = ControlPlan {
            intent: Intent {
                movement,
                manual: None,
            },
            hold_ticks: ordinary.hold_ticks,
            next_memory: ordinary.next_memory,
        };
        let mut child = WorkBudget::new(slice);
        let planned = certified_plant_inner_prefiltered(
            observation,
            history,
            settled_stop_tick,
            memory,
            &candidate,
            &mut child,
        );
        work.try_take(child.spent()).ok()?;
        if let Some(mut planned) = planned {
            planned.control.next_memory = planned
                .control
                .next_memory
                .carry_progress(ordinary.next_memory);
            return Some(planned);
        }
    }
    None
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
    certified_plant_inner_prefiltered(
        observation,
        history,
        settled_stop_tick,
        memory,
        ordinary,
        work,
    )
}

fn certified_plant_inner_prefiltered(
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
