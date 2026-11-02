use crate::cpu::{Observation, VisibleActor, WorkBudget};
use crate::game::CellId;

use super::{GuidanceV24, select_inner};
use crate::cpu::policy::{GoalModes, PressureMode, UtilityMemory};

pub(in crate::cpu::policy) fn select(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        false,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        false,
    )
}

pub(in crate::cpu::policy) fn select_v25(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        GoalModes::nearest(PressureMode::Disabled),
        false,
        false,
    )
}

pub(in crate::cpu::policy) fn select_v28(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        GoalModes::nearest(PressureMode::Baseline),
        false,
        false,
    )
}

pub(in crate::cpu::policy) fn select_v38(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
    pressure_mode: PressureMode,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        GoalModes::nearest(pressure_mode),
        true,
        false,
    )
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cpu::policy) fn select_pits(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
    pressure_mode: PressureMode,
    farm_origin_recovery: bool,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        GoalModes::nearest(pressure_mode),
        farm_origin_recovery,
        true,
    )
}

pub(in crate::cpu::policy) fn select_pits_v59(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
    farm_origin_recovery: bool,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        GoalModes::visible_yield(),
        farm_origin_recovery,
        true,
    )
}
