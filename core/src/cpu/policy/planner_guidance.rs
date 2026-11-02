use crate::cpu::{Observation, VisibleActor, WorkBudget};
use crate::game::CellId;

use super::super::{GoalModes, UtilityMemory};

#[allow(clippy::too_many_arguments)]
pub(super) fn strategic(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
    modes: GoalModes,
    failed_plant_recovery: bool,
    farm_origin_recovery: bool,
    avoid_pits: bool,
) -> Option<super::super::goals::v24::GuidanceV24> {
    if avoid_pits && modes.farm_origin == super::super::FarmOriginMode::VisibleYield {
        super::super::pit_planner::select_guidance_v59(
            observation,
            actor,
            current,
            memory,
            work,
            farm_origin_recovery,
        )
    } else if avoid_pits {
        super::super::pit_planner::select_guidance(
            observation,
            actor,
            current,
            memory,
            work,
            modes.pressure,
            farm_origin_recovery,
        )
    } else if farm_origin_recovery {
        super::super::goals::select_v38(observation, actor, current, memory, work, modes.pressure)
    } else if modes.pressure.enabled() {
        super::super::goals::select_v28(observation, actor, current, memory, work)
    } else if failed_plant_recovery {
        super::super::goals::select_v25(observation, actor, current, memory, work)
    } else {
        super::super::goals::select_v24(observation, actor, current, memory, work)
    }
}
