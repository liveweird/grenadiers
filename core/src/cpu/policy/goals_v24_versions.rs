use crate::cpu::{Observation, VisibleActor, WorkBudget};
use crate::game::CellId;

use super::{GuidanceV24, select_inner};
use crate::cpu::policy::{PressureMode, UtilityMemory};

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
        PressureMode::Disabled,
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
        PressureMode::Disabled,
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
        PressureMode::Baseline,
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
        pressure_mode,
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
        pressure_mode,
        farm_origin_recovery,
        true,
    )
}
