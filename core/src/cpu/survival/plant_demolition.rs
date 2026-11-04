use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    ControlPlan, StrategicObjectiveKind, StrategicPhase, StrategicTarget, UtilityMemory,
};
use crate::cpu::routing::{
    PlantRequest, RouteLimits, plan_lair_ordinary_plant, plan_lair_timed_weapon_plant,
};
use crate::cpu::{Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::plant_tactic::{PlantSupport, teleporter_plant_supported};
use super::{ExecutionCertificate, SurvivalPlan, compatibility::actor};

/// V64-only first-candidate demolition. It never changes an existing strategic
/// commitment and spends only the preferred candidate's original reserved slice.
#[allow(clippy::too_many_arguments)]
pub(super) fn certified_initial_demolition(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    memory: &UtilityMemory,
    ordinary: &ControlPlan,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    debug_assert!(eligible(memory, ordinary));
    let attack = ordinary.next_memory.strategic_intent()?;
    let mut prefilter = WorkBudget::new(work.remaining() / 8);
    let useful = super::plant_value::could_be_useful(observation, &mut prefilter);
    work.try_take(prefilter.spent()).ok()?;
    if !useful {
        return None;
    }
    let total = work.remaining();
    let reserve = total / 10;
    let preferred_slice = total.saturating_sub(reserve.saturating_mul(4));
    let mut child = WorkBudget::new(preferred_slice);
    let result = certify(
        observation,
        history,
        settled_stop_tick,
        ordinary,
        attack,
        &mut child,
    );
    work.try_take(child.spent()).ok()?;
    if result.is_some() {
        return result;
    }
    for movement in remaining_movements(ordinary.intent.movement) {
        let candidate = ControlPlan {
            intent: Intent {
                movement,
                manual: None,
            },
            hold_ticks: ordinary.hold_ticks,
            next_memory: ordinary.next_memory,
        };
        let mut child = WorkBudget::new(reserve);
        let planned = super::plant_tactic::certified_plant_inner_prefiltered(
            observation,
            history,
            settled_stop_tick,
            memory,
            &candidate,
            PlantSupport::Strict,
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

fn remaining_movements(first: MovementIntent) -> impl Iterator<Item = MovementIntent> {
    [
        MovementIntent::Stop,
        MovementIntent::Direction(Direction::North),
        MovementIntent::Direction(Direction::East),
        MovementIntent::Direction(Direction::South),
        MovementIntent::Direction(Direction::West),
    ]
    .into_iter()
    .filter(move |movement| *movement != first)
    .take(4)
}

pub(super) fn eligible(memory: &UtilityMemory, ordinary: &ControlPlan) -> bool {
    memory.strategic_intent().is_none()
        && ordinary.intent.manual.is_none()
        && ordinary
            .next_memory
            .strategic_intent()
            .is_some_and(|intent| {
                intent.objective == StrategicObjectiveKind::Attack
                    && matches!(intent.target, StrategicTarget::Opponent { .. })
                    && intent.phase == StrategicPhase::Approach
            })
}

fn certify(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    ordinary: &ControlPlan,
    attack: crate::cpu::policy::StrategicIntent,
    work: &mut WorkBudget,
) -> Option<SurvivalPlan> {
    if !teleporter_plant_supported(observation, PlantSupport::Strict, work) {
        return None;
    }
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    let request = PlantRequest {
        execution_tick,
        settled_stop_tick,
        first_movement: ordinary.intent.movement,
        limits: RouteLimits::default(),
    };
    let actor = actor(observation)?;
    let certificate = if actor.weapon.is_some() {
        plan_lair_timed_weapon_plant(observation, history, request, work)
    } else {
        plan_lair_ordinary_plant(observation, history, request, work)
    }
    .ok()?;
    if super::plant_value::useful_plant_for(
        observation,
        certificate.placement(),
        attack.target,
        work,
    )? {
        return Some(plan(ordinary, attack, certificate));
    }
    let geometry = super::plant_value::ObservedBlastGeometry::new(observation, work)?;
    let placement = certificate.placement();
    let block = observation
        .parts()
        .arena
        .blocks
        .iter()
        .copied()
        .find(|&cell| {
            geometry
                .visible_block_yield(placement.cell, placement.range, placement.blast, cell, work)
                .is_some_and(|yield_count| yield_count > 0)
        })?;
    let intent = crate::cpu::policy::StrategicIntent {
        objective: StrategicObjectiveKind::Farm,
        target: StrategicTarget::Block { cell: block },
        approach_cell: Some(placement.cell),
        phase: StrategicPhase::Plant,
        score: 2,
        last_distance: 0,
        ..attack
    };
    let next_memory = UtilityMemory::new(memory_key(ordinary))
        .carry_progress(ordinary.next_memory)
        .with_strategic_intent(intent);
    Some(SurvivalPlan {
        control: ControlPlan {
            intent: Intent {
                movement: ordinary.intent.movement,
                manual: Some(ManualIntent::Bomb),
            },
            hold_ticks: 1,
            next_memory,
        },
        lease: None,
        execution: Some(ExecutionCertificate::Plant(Box::new(certificate))),
    })
}

fn plan(
    ordinary: &ControlPlan,
    intent: crate::cpu::policy::StrategicIntent,
    certificate: crate::cpu::routing::PlantCertificate,
) -> SurvivalPlan {
    SurvivalPlan {
        control: ControlPlan {
            intent: Intent {
                movement: ordinary.intent.movement,
                manual: Some(ManualIntent::Bomb),
            },
            hold_ticks: 1,
            next_memory: UtilityMemory::new(memory_key(ordinary))
                .carry_progress(ordinary.next_memory)
                .mark_strategic_phase(
                    StrategicPhase::Plant,
                    intent.last_switch_reason,
                    intent.observation_tick,
                ),
        },
        lease: None,
        execution: Some(ExecutionCertificate::Plant(Box::new(certificate))),
    }
}

fn memory_key(ordinary: &ControlPlan) -> crate::cpu::MemoryKey {
    ordinary.next_memory.key()
}
