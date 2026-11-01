use crate::cpu::{Observation, WorkBudget};
use crate::game::{CellId, PickupKind};

use super::goals::{cell_index, search_map};
use super::teleporter_guidance::{
    TeleporterGuidance, guidance_for_target, reachable_approaches, recipient_actor,
};
use super::{StrategicObjectiveKind, StrategicTarget, UtilityMemory};

pub(crate) fn teleporter_guidance_v50(
    observation: &Observation,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<TeleporterGuidance> {
    let parts = observation.parts();
    let (actor, current) = recipient_actor(observation, memory, work)?;
    let map = search_map(observation, actor, current, false, work)?;
    let (distance, first_step) = super::goals::v24::reachable(observation, current, &map, work)?;

    work.try_take(u32::try_from(parts.pickups.len()).ok()?)
        .ok()?;
    let retained_target = retained_target(*memory);
    let mut retained = None;
    let mut selected = None;
    for pickup in parts
        .pickups
        .iter()
        .filter(|pickup| pickup.kind == PickupKind::Teleporter)
    {
        let approaches = reachable_approaches(observation, pickup.cell, &map, &distance, work)?;
        let best_distance = approaches
            .iter()
            .filter_map(|cell| cell_index(observation, *cell).and_then(|index| distance[index]))
            .min();
        let Some(best_distance) = best_distance else {
            continue;
        };
        if retained_target == Some(pickup.cell) {
            retained = Some((pickup.cell, approaches));
        } else {
            let key = (best_distance, pickup.cell.row, pickup.cell.column);
            if selected
                .as_ref()
                .is_none_or(|(selected_key, _, _)| key < *selected_key)
            {
                selected = Some((key, pickup.cell, approaches));
            }
        }
    }
    let (target, approaches) =
        retained.or_else(|| selected.map(|(_, target, approaches)| (target, approaches)))?;
    guidance_for_target(
        observation,
        memory,
        work,
        actor,
        current,
        target,
        approaches,
        &distance,
        &first_step,
    )
}

fn retained_target(memory: UtilityMemory) -> Option<CellId> {
    let intent = memory.strategic_intent()?;
    if intent.objective != StrategicObjectiveKind::Collect {
        return None;
    }
    match intent.target {
        StrategicTarget::Pickup {
            cell,
            kind: PickupKind::Teleporter,
        } => Some(cell),
        _ => None,
    }
}

#[cfg(test)]
#[path = "teleporter_guidance_v50_tests.rs"]
mod tests;
