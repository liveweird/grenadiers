use std::collections::VecDeque;

use crate::cpu::{Observation, VisibleActor, WorkBudget};
use crate::game::CellId;
use crate::game::arena::ARENA_CELL_COUNT;

use super::super::movement::{STEPS, offset};
use super::super::{GoalKind, GoalMemory, UtilityMemory};
use super::{Guidance, SearchMap, cell_index, search_map};

#[derive(Clone, Copy)]
struct Reached {
    cell: CellId,
    value: u8,
    distance: u16,
    first_step: CellId,
}

/// Keeps a reachable objective until it is reached, invalidated, or a
/// strictly higher-priority objective appears.
pub(in crate::cpu::policy) fn select(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<Guidance> {
    let map = search_map(observation, actor, current, work)?;
    if map.highest_value == 0 {
        return fallback_step(observation, current, &map, memory, work);
    }
    let mut queue = VecDeque::from([(current, 0_u16, current)]);
    let mut visited = [false; ARENA_CELL_COUNT];
    visited[cell_index(observation, current)?] = true;
    let mut reached = Vec::new();
    while let Some((cell, distance, first_step)) = queue.pop_front() {
        work.try_take(1).ok()?;
        let value = map.value(observation, cell);
        if distance > 0 && value > 0 {
            reached.push(Reached {
                cell,
                value,
                distance,
                first_step,
            });
        }
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            let Some(index) = cell_index(observation, next) else {
                continue;
            };
            if !visited[index] && map.transitable(observation, next) {
                visited[index] = true;
                queue.push_back((
                    next,
                    distance.saturating_add(1),
                    if cell == current { next } else { first_step },
                ));
            }
        }
    }
    if let Some(deferred) = memory.deferred_goal()
        && memory.recent_cells().contains(&Some(deferred.cell))
        && map.value(observation, deferred.cell) == deferred.priority
        && classify_goal(observation, deferred.cell) == deferred.kind
    {
        reached.retain(|candidate| candidate.cell != deferred.cell);
    }
    let Some(reachable_highest) = reached.iter().map(|candidate| candidate.value).max() else {
        return fallback_step(observation, current, &map, memory, work);
    };
    if let Some(goal) = memory.goal()
        && goal.priority >= reachable_highest
        && map.value(observation, goal.cell) == goal.priority
        && classify_goal(observation, goal.cell) == goal.kind
        && let Some(previous) = reached.iter().find(|candidate| candidate.cell == goal.cell)
    {
        return Some(Guidance {
            first_step: previous.first_step,
            goal: Some(goal),
        });
    }
    reached.sort_unstable_by_key(|candidate| {
        (
            std::cmp::Reverse(candidate.value),
            candidate.distance,
            candidate.cell.row,
            candidate.cell.column,
            candidate.first_step.row,
            candidate.first_step.column,
        )
    });
    let best_value = reached.first()?.value;
    let best_distance = reached.first()?.distance;
    let chosen = reached
        .iter()
        .take_while(|candidate| {
            candidate.value == best_value && candidate.distance == best_distance
        })
        .find(|candidate| !memory.recent_cells().contains(&Some(candidate.first_step)))
        .or_else(|| reached.first())?;
    Some(Guidance {
        first_step: chosen.first_step,
        goal: Some(GoalMemory {
            kind: classify_goal(observation, chosen.cell),
            cell: chosen.cell,
            priority: chosen.value,
        }),
    })
}

fn fallback_step(
    observation: &Observation,
    current: CellId,
    map: &SearchMap,
    memory: &UtilityMemory,
    work: &mut WorkBudget,
) -> Option<Guidance> {
    let mut choices = Vec::new();
    for (_, dc, dr) in STEPS {
        work.try_take(1).ok()?;
        if let Some(next) = offset(current, dc, dr)
            && map.transitable(observation, next)
        {
            choices.push(next);
        }
    }
    choices.sort_unstable_by_key(|cell| {
        (
            memory.recent_cells().contains(&Some(*cell)),
            cell.row,
            cell.column,
        )
    });
    Some(Guidance {
        first_step: *choices.first()?,
        goal: None,
    })
}

fn classify_goal(observation: &Observation, cell: CellId) -> GoalKind {
    if observation
        .parts()
        .pickups
        .iter()
        .any(|pickup| pickup.cell == cell)
    {
        GoalKind::Pickup
    } else if observation
        .parts()
        .arena
        .blocks
        .iter()
        .any(|block| block.column.abs_diff(cell.column) + block.row.abs_diff(cell.row) == 1)
    {
        GoalKind::Farming
    } else {
        GoalKind::Combat
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::policy::tests::observation;
    use crate::cpu::{MemoryKey, ObservedActor, WorkBudget};
    use crate::game::{CellId, Position, Terrain};

    use super::*;

    #[test]
    fn sole_attained_farming_goal_does_not_rebuild_an_ab_cycle() {
        let attained = CellId::new(7, 7);
        let mut observed = observation(attained)
            .map_parts(|parts| {
                parts.arena.blocks.push(CellId::new(8, 7));
                for cell in [CellId::new(8, 6), CellId::new(9, 7), CellId::new(8, 8)] {
                    let index = parts.arena.index(cell).unwrap();
                    parts.arena.terrain[index] = Terrain::Wall;
                }
            })
            .unwrap();
        let key = MemoryKey {
            world: observed.parts().world,
            cpu_seat: observed.parts().recipient,
        };
        let mut memory = UtilityMemory::new(key).with_progress(
            attained,
            GoalMemory {
                kind: GoalKind::Farming,
                cell: attained,
                priority: 2,
            },
        );
        let mut settled = vec![attained];
        for _ in 0..8 {
            let current = *settled.last().unwrap();
            let ObservedActor::Visible(actor) = &observed.parts().actors[1] else {
                unreachable!()
            };
            let guidance = select(
                &observed,
                actor,
                current,
                &memory,
                &mut WorkBudget::new(65_536),
            )
            .unwrap();
            memory = match guidance.goal {
                Some(goal) => memory.with_progress(current, goal),
                None => memory.without_guidance(current),
            };
            let next = guidance.first_step;
            settled.push(next);
            observed = observed
                .map_parts(|parts| {
                    let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                        unreachable!()
                    };
                    actor.position = Position::center(next);
                })
                .unwrap();
        }
        assert!(
            settled
                .iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 2
        );
        assert!(
            settled.windows(6).all(|cells| !(cells[0] == cells[2]
                && cells[2] == cells[4]
                && cells[1] == cells[3]
                && cells[3] == cells[5])),
            "{settled:?}"
        );
    }

    #[test]
    fn deferred_non_pickup_goal_expires_after_recent_ring_eviction() {
        let attained = CellId::new(7, 7);
        let settled = [
            CellId::new(6, 7),
            CellId::new(6, 6),
            CellId::new(7, 6),
            CellId::new(7, 5),
            CellId::new(6, 5),
            CellId::new(5, 5),
            CellId::new(5, 6),
            CellId::new(5, 7),
        ];
        let mut observed = observation(attained)
            .map_parts(|parts| {
                parts.arena.blocks.push(CellId::new(8, 7));
                for cell in [CellId::new(8, 6), CellId::new(9, 7), CellId::new(8, 8)] {
                    let index = parts.arena.index(cell).unwrap();
                    parts.arena.terrain[index] = Terrain::Wall;
                }
            })
            .unwrap();
        let key = MemoryKey {
            world: observed.parts().world,
            cpu_seat: observed.parts().recipient,
        };
        let goal = GoalMemory {
            kind: GoalKind::Farming,
            cell: attained,
            priority: 2,
        };
        let mut memory = UtilityMemory::new(key)
            .with_progress(attained, goal)
            .without_guidance(attained);
        for cell in settled {
            memory = memory.without_guidance(cell);
        }
        let current = *settled.last().unwrap();
        observed = observed
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[1] else {
                    unreachable!()
                };
                actor.position = Position::center(current);
            })
            .unwrap();
        let ObservedActor::Visible(actor) = &observed.parts().actors[1] else {
            unreachable!()
        };
        let guidance = select(
            &observed,
            actor,
            current,
            &memory,
            &mut WorkBudget::new(65_536),
        )
        .unwrap();
        assert_eq!(guidance.goal.map(|goal| goal.cell), Some(attained));
    }
}
