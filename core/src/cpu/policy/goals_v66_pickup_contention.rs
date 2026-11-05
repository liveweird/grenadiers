use std::collections::VecDeque;

use crate::cpu::{Observation, ObservedActor, VisibleActor, WorkBudget};
use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::{EffectKind, Equipment, OilBarrelState, PickupKind, Terrain};

use super::{Candidate, DistanceMap, cell_index};
use crate::cpu::policy::StrategicObjectiveKind;
use crate::cpu::policy::movement::{STEPS, offset};
use crate::cpu::policy::pit_avoidance::PitMask;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Race {
    Uncontested,
    CpuClearlyFirst,
    Ambiguous,
}

pub(super) fn promote_fresh(
    observation: &Observation,
    cpu: &VisibleActor,
    baseline: Candidate,
    candidates: &[Candidate],
    pits: &PitMask,
    work: &mut WorkBudget,
) -> Candidate {
    if baseline.objective != StrategicObjectiveKind::Collect {
        return baseline;
    }
    let allowance = work.remaining().min(2_048).min(work.remaining() / 16);
    let mut child = WorkBudget::new(allowance);
    let promoted = analyze(observation, cpu, baseline, candidates, pits, &mut child);
    let _ = work.try_take(child.spent());
    promoted.unwrap_or(baseline)
}

fn analyze(
    observation: &Observation,
    cpu: &VisibleActor,
    baseline: Candidate,
    candidates: &[Candidate],
    pits: &PitMask,
    work: &mut WorkBudget,
) -> Option<Candidate> {
    let walkable = physical_walkability(observation, work)?;
    work.try_take(u32::try_from(observation.parts().actors.len()).ok()?)
        .ok()?;
    if !eligible_cpu(cpu, observation) {
        return None;
    }
    let mut rivals = Vec::new();
    for observed in &observation.parts().actors {
        match observed {
            ObservedActor::Visible(rival) if rival.seat == cpu.seat || !rival.alive => {}
            ObservedActor::Visible(rival) if eligible_rival(cpu, rival, observation) => {
                rivals.push(rival);
            }
            ObservedActor::Visible(_) | ObservedActor::Hidden { alive: true, .. } => return None,
            ObservedActor::Hidden { alive: false, .. } => {}
        }
    }
    rivals.sort_unstable_by_key(|rival| rival.seat);
    if rivals.is_empty() {
        return Some(baseline);
    }
    let maps = rivals
        .into_iter()
        .map(|rival| rival_distances(observation, rival, pits, &walkable, work))
        .collect::<Option<Vec<_>>>()?;
    if race(observation, baseline, &maps, work)? != Race::Uncontested {
        return Some(baseline);
    }
    for candidate in candidates.iter().copied() {
        work.try_take(1).ok()?;
        if candidate.objective == StrategicObjectiveKind::Collect
            && candidate.score == baseline.score
            && candidate.distance == baseline.distance
            && race(observation, candidate, &maps, work)? == Race::CpuClearlyFirst
        {
            return Some(candidate);
        }
    }
    Some(baseline)
}

fn race(
    observation: &Observation,
    candidate: Candidate,
    maps: &[DistanceMap],
    work: &mut WorkBudget,
) -> Option<Race> {
    let index = cell_index(observation, candidate.approach)?;
    work.try_take(u32::try_from(maps.len()).ok()?).ok()?;
    let Some(rival_distance) = maps.iter().filter_map(|map| map[index]).min() else {
        return Some(Race::Uncontested);
    };
    let margin = rival_distance.saturating_sub(candidate.distance);
    Some(if rival_distance < candidate.distance.saturating_add(2) {
        Race::Ambiguous
    } else if margin <= 4 {
        Race::CpuClearlyFirst
    } else {
        Race::Uncontested
    })
}

fn rival_distances(
    observation: &Observation,
    rival: &VisibleActor,
    _pits: &PitMask,
    walkable: &[bool; ARENA_CELL_COUNT],
    work: &mut WorkBudget,
) -> Option<DistanceMap> {
    let start = rival.position.cell()?;
    let mut distance: DistanceMap = [None; ARENA_CELL_COUNT];
    distance[cell_index(observation, start)?] = Some(0);
    let mut queue = VecDeque::from([start]);
    while let Some(cell) = queue.pop_front() {
        work.try_take(1).ok()?;
        let next_distance = distance[cell_index(observation, cell)?]?.saturating_add(1);
        for (_, dc, dr) in STEPS {
            let Some(next) = offset(cell, dc, dr) else {
                continue;
            };
            let Some(index) = cell_index(observation, next) else {
                continue;
            };
            if distance[index].is_none() && walkable[index] {
                distance[index] = Some(next_distance);
                queue.push_back(next);
            }
        }
    }
    Some(distance)
}

fn physical_walkability(
    observation: &Observation,
    work: &mut WorkBudget,
) -> Option<[bool; ARENA_CELL_COUNT]> {
    let parts = observation.parts();
    let scans = parts
        .arena
        .terrain
        .len()
        .checked_add(parts.arena.blocks.len())?
        .checked_add(parts.oil.len())?
        .checked_add(parts.pickups.len())?;
    work.try_take(u32::try_from(scans).ok()?).ok()?;
    if parts.pickups.iter().any(|pickup| {
        !matches!(
            pickup.kind,
            PickupKind::BombCapacity | PickupKind::BlastRange
        )
    }) {
        return None;
    }
    let mut walkable = [false; ARENA_CELL_COUNT];
    for (index, terrain) in parts.arena.terrain.iter().enumerate() {
        walkable[index] = *terrain == Terrain::Floor;
    }
    for &block in &parts.arena.blocks {
        walkable[cell_index(observation, block)?] = false;
    }
    for oil in &parts.oil {
        if oil.state == OilBarrelState::Unlit {
            walkable[cell_index(observation, oil.cell)?] = false;
        }
    }
    Some(walkable)
}

fn eligible_cpu(cpu: &VisibleActor, observation: &Observation) -> bool {
    !recently_teleported(cpu, observation)
        && cpu.equipment != Some(Equipment::WallPass)
        && !has_race_effect(cpu)
}

fn eligible_rival(cpu: &VisibleActor, rival: &VisibleActor, observation: &Observation) -> bool {
    rival.alive
        && !rival.voluntary_actions_frozen
        && rival.speed == cpu.speed
        && !recently_teleported(rival, observation)
        && rival.equipment != Some(Equipment::WallPass)
        && !has_race_effect(rival)
}

fn recently_teleported(actor: &VisibleActor, observation: &Observation) -> bool {
    actor
        .last_teleport_tick
        .is_some_and(|tick| observation.parts().tick.0.saturating_sub(tick.0) <= 6)
}

fn has_race_effect(actor: &VisibleActor) -> bool {
    actor
        .effects
        .iter()
        .any(|effect| matches!(effect.kind, EffectKind::SlowFoot | EffectKind::Stopwatch))
}

#[cfg(test)]
#[path = "goals_v66_pickup_contention_tests.rs"]
mod tests;
