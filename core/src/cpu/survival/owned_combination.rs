use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    ControlPlan, PolicyError, StrategicObjectiveKind, StrategicTarget, UtilityMemory,
};
use crate::cpu::routing::{PlantRequest, RouteLimits, plan_lair_reactive_ordinary_plant};
use crate::cpu::{Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::Tick;

use super::reactive_assumptions::ReactivePlantAssumptions;
use super::reactive_purpose::{ReactivePlantPurpose, ReactivePurposeContext};
use super::{ExecutionCertificate, LairSurvivalPlan, SurvivalPlan, TerminalRefugeHandoff};

pub fn plan_hard_combination_v67_with_handoff(
    observation: &Observation,
    prospective_memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    handoff: &TerminalRefugeHandoff,
    work: &mut WorkBudget,
) -> Result<Option<LairSurvivalPlan>, PolicyError> {
    let parts = observation.parts();
    if prospective_memory.key().world != parts.world
        || prospective_memory.key().cpu_seat != parts.recipient
        || history.key() != prospective_memory.key()
    {
        return Err(PolicyError::IdentityMismatch);
    }
    if parts.rules.version != 10
        || parts.rules.punch_activation_policy != crate::game::PunchActivationPolicy::ContactV2
    {
        return Err(PolicyError::InvalidObservation);
    }
    let Some(assumptions) = ReactivePlantAssumptions::certify_owned_combination(
        observation,
        history,
        settled_stop_tick,
        handoff,
        work,
    ) else {
        return Ok(None);
    };
    let Some(intent) = prospective_memory.strategic_intent() else {
        return Ok(None);
    };
    if !matches!(
        (intent.objective, intent.target),
        (StrategicObjectiveKind::Farm, StrategicTarget::Block { .. })
            | (
                StrategicObjectiveKind::Attack,
                StrategicTarget::Opponent { .. }
            )
    ) {
        return Ok(None);
    }
    let Some(context) = ReactivePurposeContext::new(observation, intent.target, work) else {
        return Ok(None);
    };
    let Some(execution_tick) = parts.tick.0.checked_add(7).map(Tick) else {
        return Ok(None);
    };
    let Ok(plant) = plan_lair_reactive_ordinary_plant(
        observation,
        history,
        PlantRequest {
            execution_tick,
            settled_stop_tick,
            first_movement: MovementIntent::Stop,
            limits: RouteLimits::default(),
        },
        work,
    ) else {
        return Ok(None);
    };
    let placement = *plant.placement();
    let Some(purpose) = ReactivePlantPurpose::select(
        observation,
        &context,
        placement,
        plant.route().evolution(),
        work,
    ) else {
        return Ok(None);
    };
    let Ok(contact) = crate::cpu::MobContactLease::certify_route(observation, plant.route(), work)
    else {
        return Ok(None);
    };
    let Some(certificate) = super::ReactivePlantCertificate::new_owned_combination(
        plant,
        assumptions.with_contact(contact.clone()),
        purpose,
        handoff,
    ) else {
        return Ok(None);
    };
    let execution = ExecutionCertificate::ReactivePlant(Box::new(certificate));
    if !handoff.accepts_replacement_charged(&execution, work) {
        return Ok(None);
    }
    let next_memory = super::reactive_plant::mark_plant(*prospective_memory, intent);
    Ok(Some(LairSurvivalPlan {
        survival: SurvivalPlan {
            control: ControlPlan {
                intent: Intent {
                    movement: MovementIntent::Stop,
                    manual: Some(ManualIntent::Bomb),
                },
                hold_ticks: 1,
                next_memory,
            },
            lease: None,
            execution: Some(execution),
        },
        contact: Some(contact),
    }))
}

pub fn plan_hard_combination_v69_with_handoff(
    observation: &Observation,
    prospective_memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    handoff: &TerminalRefugeHandoff,
    work: &mut WorkBudget,
) -> Result<Option<LairSurvivalPlan>, PolicyError> {
    plan_hard_combination_v67_with_handoff(
        observation,
        prospective_memory,
        history,
        settled_stop_tick,
        handoff,
        work,
    )
}
