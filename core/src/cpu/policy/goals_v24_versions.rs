use crate::cpu::{Observation, VisibleActor, WorkBudget};
use crate::game::CellId;

use super::{GuidanceV24, select_inner};
use crate::cpu::policy::UtilityMemory;

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
        false,
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
        false,
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
        true,
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
    hard: bool,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        hard,
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
    hard_tactics: bool,
    farm_origin_recovery: bool,
) -> Option<GuidanceV24> {
    select_inner(
        observation,
        actor,
        current,
        memory,
        work,
        true,
        hard_tactics,
        farm_origin_recovery,
        true,
    )
}
