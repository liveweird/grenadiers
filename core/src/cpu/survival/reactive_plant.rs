use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, StrategicPhase, StrategicTarget, UtilityMemory};
use crate::cpu::routing::{
    PlantCertificate, PlantRequest, RouteAssurance, RouteLimits, plan_lair_reactive_ordinary_plant,
};
use crate::cpu::{CpuRng, Intent, ManualIntent, MovementIntent, Observation, WorkBudget};
use crate::game::{Direction, Tick};

use super::reactive_assumptions::ReactivePlantAssumptions;
use super::reactive_purpose::{ReactivePlantPurpose, ReactivePurposeContext};
use super::{ExecutionCertificate, LairSurvivalPlan, PlantEscapeOrigin, SurvivalPlan};

pub(super) use super::reactive_assumptions::REACTIVE_GUARD_ALLOWANCE;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReactivePlantCertificate {
    plant: PlantCertificate,
    assumptions: ReactivePlantAssumptions,
    purpose: ReactivePlantPurpose,
}

impl ReactivePlantCertificate {
    fn new(
        plant: PlantCertificate,
        assumptions: ReactivePlantAssumptions,
        purpose: ReactivePlantPurpose,
    ) -> Option<Self> {
        let RouteAssurance::ReactiveStationary { bombs } = plant.route().assurance() else {
            return None;
        };
        if bombs
            .iter()
            .any(|id| !assumptions.eligible_bombs.contains(id))
        {
            return None;
        }
        assumptions.contact.as_ref()?;
        Some(Self {
            plant,
            assumptions,
            purpose,
        })
    }

    pub(crate) const fn plant(&self) -> &PlantCertificate {
        &self.plant
    }

    pub(super) fn into_parts(self) -> (PlantCertificate, ReactivePlantAssumptions) {
        (self.plant, self.assumptions)
    }

    pub(crate) fn strategic_plant_still_targets(
        &self,
        observation: &Observation,
        intent: crate::cpu::policy::StrategicIntent,
        work: &mut WorkBudget,
    ) -> bool {
        self.purpose
            .still_targets(observation, intent, self.plant.placement(), work)
    }

    pub(crate) const fn block_contribution(&self) -> crate::game::CellId {
        self.purpose.block()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn plan_hard_reactive_with_modes(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    goal_modes: GoalModes,
) -> Result<LairSurvivalPlan, PolicyError> {
    if origin.is_some() {
        return fallback(
            observation,
            memory,
            history,
            settled_stop_tick,
            origin,
            rng,
            work,
            goal_modes,
        );
    }
    let entry_rng = rng.clone();
    let mut guard_work = WorkBudget::new(work.remaining().min(REACTIVE_GUARD_ALLOWANCE));
    let mut assumptions =
        ReactivePlantAssumptions::certify(observation, history, settled_stop_tick, &mut guard_work);
    let escape_preflight = if assumptions.is_none() {
        ReactivePlantAssumptions::certify_escape(
            observation,
            history,
            settled_stop_tick,
            &mut guard_work,
        )
        .is_some()
    } else {
        false
    };
    work.try_take(guard_work.spent())
        .map_err(|_| PolicyError::InvalidObservation)?;
    if assumptions.is_none() && !escape_preflight {
        return fallback(
            observation,
            memory,
            history,
            settled_stop_tick,
            None,
            rng,
            work,
            goal_modes,
        );
    }
    let fallback_allowance = work.remaining().min(32_768).min(work.remaining() / 2);
    let mut fallback_work = WorkBudget::new(fallback_allowance);
    let mut fallback_rng = entry_rng.clone();
    let fallback = fallback(
        observation,
        memory,
        history,
        settled_stop_tick,
        None,
        &mut fallback_rng,
        &mut fallback_work,
        goal_modes,
    )?;
    work.try_take(fallback_work.spent())
        .map_err(|_| PolicyError::InvalidObservation)?;
    *rng = fallback_rng.clone();
    if fallback.survival.control.intent.manual.is_some()
        || fallback
            .survival
            .execution
            .as_ref()
            .and_then(ExecutionCertificate::manual_intent)
            .is_some()
    {
        return Ok(fallback);
    }
    match super::reactive_escape::certify(
        observation,
        history,
        settled_stop_tick,
        &fallback.survival,
        work,
    ) {
        super::reactive_escape::ReactiveEscapeAttempt::Certified(survival, contact) => {
            return Ok(LairSurvivalPlan {
                survival: *survival,
                contact: Some(contact),
            });
        }
        super::reactive_escape::ReactiveEscapeAttempt::Failed => return Ok(fallback),
        super::reactive_escape::ReactiveEscapeAttempt::Ineligible => {}
    }
    let Some(assumptions) = assumptions.take() else {
        return Ok(fallback);
    };
    let mut candidate = fallback.survival.clone();
    let mut success_rng = fallback_rng;
    let mut target = guidance_target(candidate.control.next_memory.strategic_intent());
    if target.is_none()
        && let Some(incoming) = memory
            .strategic_intent()
            .filter(|intent| guidance_target(Some(*intent)).is_some())
    {
        target = Some(incoming.target);
        candidate.control.next_memory = candidate
            .control
            .next_memory
            .with_strategic_intent(incoming);
    }
    if target.is_none() {
        let allowance = work.remaining().min(4_096).min(work.remaining() / 8);
        let mut goal_work = WorkBudget::new(allowance);
        let mut goal_rng = entry_rng;
        let goal = if goal_modes.pickup == crate::cpu::policy::PickupMode::VisibleContention {
            crate::cpu::policy::plan_hard_pickup_contention_v66(
                observation,
                memory,
                &mut goal_rng,
                &mut goal_work,
            )
        } else {
            crate::cpu::policy::plan_hard_farm_yield_v59(
                observation,
                memory,
                &mut goal_rng,
                &mut goal_work,
            )
        };
        if let Ok(goal) = goal {
            target = guidance_target(goal.next_memory.strategic_intent());
            if target.is_some() {
                candidate.control = goal;
                candidate.lease = None;
                candidate.execution = None;
                success_rng = goal_rng;
            }
        }
        work.try_take(goal_work.spent())
            .map_err(|_| PolicyError::InvalidObservation)?;
    }
    let Some(target) = target else {
        return Ok(fallback);
    };
    let Some(purpose_context) = ReactivePurposeContext::new(observation, target, work) else {
        return Ok(fallback);
    };
    let movements = candidate_movements(candidate.control.intent.movement);
    let total = work.remaining();
    let reserve = total / 10;
    for (index, movement) in movements.into_iter().enumerate() {
        let allowance = if index == 0 {
            total.saturating_sub(reserve.saturating_mul(4))
        } else {
            reserve
        };
        let mut child = WorkBudget::new(allowance);
        let planned = certify_candidate(
            observation,
            history,
            settled_stop_tick,
            &candidate,
            movement,
            &purpose_context,
            assumptions.clone(),
            &mut child,
        );
        if work.try_take(child.spent()).is_err() {
            return Ok(fallback);
        }
        if let Some((survival, contact)) = planned {
            *rng = success_rng;
            return Ok(LairSurvivalPlan {
                survival,
                contact: Some(contact),
            });
        }
    }
    Ok(fallback)
}

#[allow(clippy::too_many_arguments)]
fn fallback(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    goal_modes: GoalModes,
) -> Result<LairSurvivalPlan, PolicyError> {
    super::corner_refuges::require_contact_punch(observation)?;
    super::refuge_guidance_policy::plan_policy_v64(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        goal_modes,
    )
}

fn guidance_target(intent: Option<crate::cpu::policy::StrategicIntent>) -> Option<StrategicTarget> {
    intent
        .filter(|intent| {
            matches!(
                intent.phase,
                StrategicPhase::Approach | StrategicPhase::Reengage
            )
        })
        .map(|intent| intent.target)
        .filter(|target| {
            matches!(
                target,
                StrategicTarget::Block { .. } | StrategicTarget::Opponent { .. }
            )
        })
}

fn candidate_movements(first: MovementIntent) -> [MovementIntent; 5] {
    let mut values = [first; 5];
    let mut len = 1;
    for value in [
        MovementIntent::Stop,
        MovementIntent::Direction(Direction::North),
        MovementIntent::Direction(Direction::East),
        MovementIntent::Direction(Direction::South),
        MovementIntent::Direction(Direction::West),
    ] {
        if !values[..len].contains(&value) {
            values[len] = value;
            len += 1;
        }
    }
    values
}

#[allow(clippy::too_many_arguments)]
fn certify_candidate(
    observation: &Observation,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    fallback: &SurvivalPlan,
    movement: MovementIntent,
    purpose_context: &ReactivePurposeContext,
    assumptions: ReactivePlantAssumptions,
    work: &mut WorkBudget,
) -> Option<(SurvivalPlan, crate::cpu::MobContactLease)> {
    let execution_tick = observation.parts().tick.0.checked_add(7).map(Tick)?;
    let plant = plan_lair_reactive_ordinary_plant(
        observation,
        history,
        PlantRequest {
            execution_tick,
            settled_stop_tick,
            first_movement: movement,
            limits: RouteLimits::default(),
        },
        work,
    )
    .ok()?;
    let placement = *plant.placement();
    let purpose = ReactivePlantPurpose::select(
        observation,
        purpose_context,
        placement,
        plant.route().evolution(),
        work,
    )?;
    let contact =
        crate::cpu::MobContactLease::certify_route(observation, plant.route(), work).ok()?;
    let certificate =
        ReactivePlantCertificate::new(plant, assumptions.with_contact(contact.clone()), purpose)?;
    let mut next_memory = fallback.control.next_memory;
    if let Some(intent) = next_memory.strategic_intent() {
        next_memory = mark_plant(next_memory, intent);
    }
    Some((
        SurvivalPlan {
            control: crate::cpu::policy::ControlPlan {
                intent: Intent {
                    movement,
                    manual: Some(ManualIntent::Bomb),
                },
                hold_ticks: 1,
                next_memory,
            },
            lease: None,
            execution: Some(ExecutionCertificate::ReactivePlant(Box::new(certificate))),
        },
        contact,
    ))
}

pub(super) fn mark_plant(
    memory: UtilityMemory,
    intent: crate::cpu::policy::StrategicIntent,
) -> UtilityMemory {
    memory.mark_strategic_phase(
        StrategicPhase::Plant,
        intent.last_switch_reason,
        intent.observation_tick,
    )
}
