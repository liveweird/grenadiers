use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{GoalModes, PolicyError, PressureMode, UtilityMemory};
use crate::cpu::{CpuRng, Observation, WorkBudget};
use crate::game::Tick;

use super::contact_plant::ContactTopology;
use super::joint_refuge::FarmRefugeMode;
use super::steering::RefugeGuidance;
use super::{LairSurvivalPlan, PlantEscapeOrigin};

#[allow(clippy::too_many_arguments)]
pub fn plan_corner_refuges_v56_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::Disabled),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_corner_refuges_v57_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::Baseline),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_exit_pressure_v58_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::ExitCoverage),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_farm_yield_v59_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::visible_yield(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_contact_punch_v60_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    require_contact_punch(observation)?;
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::nearest(PressureMode::Disabled),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn plan_hard_contact_punch_v61_with_origin(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Result<LairSurvivalPlan, PolicyError> {
    require_contact_punch(observation)?;
    plan(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        GoalModes::visible_yield(),
    )
}

fn require_contact_punch(observation: &Observation) -> Result<(), PolicyError> {
    (observation.parts().rules.version == 10
        && observation.parts().rules.punch_activation_policy
            == crate::game::PunchActivationPolicy::ContactV2)
        .then_some(())
        .ok_or(PolicyError::InvalidObservation)
}

#[allow(clippy::too_many_arguments)]
fn plan(
    observation: &Observation,
    memory: &UtilityMemory,
    history: &ObservedHazardMemory,
    settled_stop_tick: Tick,
    origin: Option<&PlantEscapeOrigin>,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
    goal_modes: GoalModes,
) -> Result<LairSurvivalPlan, PolicyError> {
    super::refuge_guidance::plan_policy(
        observation,
        memory,
        history,
        settled_stop_tick,
        origin,
        rng,
        work,
        true,
        goal_modes,
        goal_modes.pressure.enabled(),
        true,
        RefugeGuidance::CENTER_THREATENED_ROOT,
        FarmRefugeMode::BlockedAway,
        true,
        ContactTopology::BlockedBirths,
        true,
    )
}

#[cfg(test)]
mod tests {
    use crate::cpu::forecast::ObservedHazardMemory;
    use crate::cpu::policy::UtilityMemory;
    use crate::cpu::{CpuRng, ManualIntent, ObservedActor, WorkBudget};
    use crate::game::{Equipment, PunchActivationPolicy};

    use super::*;

    #[test]
    fn contact_wrappers_require_typed_policy_and_never_request_equipment() {
        let basis = super::super::farm_yield_fixture::collected_basis_at(
            super::super::farm_yield_fixture::START,
        );
        let mut work = WorkBudget::new(super::super::WORK_ALLOWANCE);
        assert_eq!(
            plan_contact_punch_v60_with_origin(
                &basis.observation,
                &UtilityMemory::new(basis.key),
                &basis.history,
                basis.observation.parts().tick,
                None,
                &mut CpuRng::from_seed(basis.key, [60; 32]),
                &mut work,
            )
            .unwrap_err(),
            PolicyError::InvalidObservation
        );

        let observation = basis
            .observation
            .map_parts(|parts| {
                parts.rules.punch_activation_policy = PunchActivationPolicy::ContactV2;
                let recipient = parts.recipient;
                let Some(ObservedActor::Visible(actor)) = parts
                    .actors
                    .iter_mut()
                    .find(|actor| actor.seat() == recipient)
                else {
                    unreachable!()
                };
                actor.equipment = Some(Equipment::Punch);
            })
            .unwrap();
        let mut history = ObservedHazardMemory::new(basis.key);
        history.observe(&observation).unwrap();
        let planned = plan_contact_punch_v60_with_origin(
            &observation,
            &UtilityMemory::new(basis.key),
            &history,
            observation.parts().tick,
            None,
            &mut CpuRng::from_seed(basis.key, [60; 32]),
            &mut WorkBudget::new(super::super::WORK_ALLOWANCE),
        )
        .unwrap();
        assert_ne!(
            planned.survival.control.intent.manual,
            Some(ManualIntent::Equipment)
        );
        assert!(
            planned
                .survival
                .execution
                .as_ref()
                .is_none_or(|execution| execution.punch().is_none())
        );
    }
}
