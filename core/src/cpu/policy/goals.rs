use std::cmp::Ordering;
use std::collections::VecDeque;

use crate::cpu::{CpuRng, Observation, ObservedActor, VisibleActor, WorkBudget};
use crate::game::arena::ARENA_CELL_COUNT;
use crate::game::effects::GOLDEN_FLAME_RANGE;
use crate::game::{CellId, EffectKind, Equipment, PickupKind, SpeedLevel, Terrain};

use super::GoalMemory;
use super::equipment::classify as equipment_pickup;
pub(super) use super::equipment::supported_weapon;
use super::movement::{STEPS, offset};

#[path = "goals_v23.rs"]
mod v23;
pub(in crate::cpu::policy) use v23::select as select_v23;

#[path = "goals_v24.rs"]
mod v24;
#[cfg(test)]
pub(in crate::cpu::policy) use v24::GuidanceV24;
pub(in crate::cpu::policy) use v24::{select as select_v24, select_v25, select_v28};

#[path = "goal_pressure.rs"]
mod pressure;

#[path = "goal_cells.rs"]
mod cells;
pub(super) use cells::{cell_index, legal_opening};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Guidance {
    pub first_step: CellId,
    pub goal: Option<GoalMemory>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Candidate {
    cell: CellId,
    value: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Reached {
    candidate: Candidate,
    distance: u16,
    first_step: CellId,
}

pub(super) struct SearchMap {
    values: [u8; ARENA_CELL_COUNT],
    blocked: [bool; ARENA_CELL_COUNT],
    forbidden: [bool; ARENA_CELL_COUNT],
    pub(super) highest_value: u8,
}

/// Finds one reachable static goal and its first hop. This is guidance only:
/// callers still apply immediate or timed hazard checks to every movement.
pub(super) fn select(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    rng: &mut CpuRng,
    work: &mut WorkBudget,
) -> Option<Guidance> {
    let map = search_map(observation, actor, current, true, work)?;
    if map.highest_value == 0 {
        return fallback_step(observation, current, &map, rng, work);
    }
    let mut queue = VecDeque::from([(current, 0_u16, current)]);
    let mut visited = [false; ARENA_CELL_COUNT];
    visited[cell_index(observation, current)?] = true;
    let mut reached = Vec::new();
    let mut winning_distance = None;

    while let Some((cell, distance, first_step)) = queue.pop_front() {
        if work.try_take(1).is_err() {
            return None;
        }
        if winning_distance.is_some_and(|winning| distance > winning) {
            break;
        }
        let index = cell_index(observation, cell)?;
        let value = map.values[index];
        if distance > 0 && value > 0 {
            reached.push(Reached {
                candidate: Candidate { cell, value },
                distance,
                first_step,
            });
            if value == map.highest_value {
                winning_distance.get_or_insert(distance);
            }
        }
        if winning_distance.is_some() {
            continue;
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
    if reached.is_empty() {
        fallback_step(observation, current, &map, rng, work)
    } else {
        choose_reached(reached, rng)
    }
}

pub(super) fn transit_allowed(
    observation: &Observation,
    actor: &VisibleActor,
    cell: CellId,
) -> bool {
    let parts = observation.parts();
    let Some(index) = cell_index(observation, cell) else {
        return false;
    };
    parts.arena.terrain.get(index) == Some(&Terrain::Floor)
        && (crate::cpu::capabilities::can_pass_known_soft_blocks(observation)
            || !parts.arena.blocks.contains(&cell))
        && !parts.oil.iter().any(|oil| oil.cell == cell)
        && parts
            .pickups
            .iter()
            .find(|pickup| pickup.cell == cell)
            .is_none_or(|pickup| pickup_transitable(actor, pickup.kind))
}

pub(super) fn search_map(
    observation: &Observation,
    actor: &VisibleActor,
    current: CellId,
    pressure: bool,
    work: &mut WorkBudget,
) -> Option<SearchMap> {
    let parts = observation.parts();
    let actor_scans = if matches!(parts.rules.version, 9 | 10) {
        2
    } else {
        1
    };
    let actor_scan = parts.actors.len().checked_mul(actor_scans)?;
    let scan_work = parts
        .pickups
        .len()
        .checked_add(parts.arena.blocks.len())?
        .checked_add(parts.arena.blocks.len())?
        .checked_add(parts.oil.len())?
        .checked_add(actor_scan)?;
    work.try_take(u32::try_from(scan_work).ok()?).ok()?;
    let mut map = SearchMap {
        values: [0; ARENA_CELL_COUNT],
        blocked: [false; ARENA_CELL_COUNT],
        forbidden: [false; ARENA_CELL_COUNT],
        highest_value: 0,
    };
    for &block in &parts.arena.blocks {
        map.blocked[cell_index(observation, block)?] = true;
    }
    for pickup in &parts.pickups {
        let index = cell_index(observation, pickup.cell)?;
        map.forbidden[index] = !pickup_transitable(actor, pickup.kind);
        if let Some(value) = pickup_value(observation, actor, pickup.kind) {
            map.add_goal(index, value);
        }
    }
    for &block in &parts.arena.blocks {
        for (_, dc, dr) in STEPS {
            if let Some(cell) = offset(block, dc, dr)
                && cell != current
                && let Some(index) = cell_index(observation, cell)
                && legal_opening(observation, cell)
            {
                map.add_goal(index, 2);
            }
        }
    }
    for oil in &parts.oil {
        map.forbidden[cell_index(observation, oil.cell)?] = true;
    }
    for observed in &parts.actors {
        if let ObservedActor::Visible(rival) = observed
            && rival.seat != parts.recipient
            && rival.alive
            && let Some(cell) = rival.position.cell()
        {
            let index = cell_index(observation, cell)?;
            if matches!(parts.rules.version, 9 | 10) {
                map.forbidden[index] = true;
            } else {
                map.add_goal(index, 1);
            }
        }
    }
    if pressure && matches!(parts.rules.version, 9 | 10) {
        for observed in &parts.actors {
            if let ObservedActor::Visible(rival) = observed
                && rival.seat != parts.recipient
                && rival.alive
            {
                let mut pressure_work = WorkBudget::new(work.remaining() / 2);
                let _ = pressure::add(
                    observation,
                    actor,
                    rival.position,
                    &mut map,
                    &mut pressure_work,
                );
                work.try_take(pressure_work.spent()).ok()?;
            }
        }
    }
    Some(map)
}

fn choose_reached(mut reached: Vec<Reached>, rng: &mut CpuRng) -> Option<Guidance> {
    reached.sort_unstable_by(|left, right| compare_reached(*left, *right));
    let best = *reached.first()?;
    let tied = reached
        .iter()
        .take_while(|candidate| {
            candidate.candidate.value == best.candidate.value && candidate.distance == best.distance
        })
        .count();
    let chosen = if tied > 1 {
        rng.bounded_u32(tied as u32).ok()? as usize
    } else {
        0
    };
    Some(Guidance {
        first_step: reached[chosen].first_step,
        goal: None,
    })
}

fn compare_reached(left: Reached, right: Reached) -> Ordering {
    right
        .candidate
        .value
        .cmp(&left.candidate.value)
        .then_with(|| left.distance.cmp(&right.distance))
        .then_with(|| left.candidate.cell.row.cmp(&right.candidate.cell.row))
        .then_with(|| left.candidate.cell.column.cmp(&right.candidate.cell.column))
        .then_with(|| left.first_step.row.cmp(&right.first_step.row))
        .then_with(|| left.first_step.column.cmp(&right.first_step.column))
}

fn fallback_step(
    observation: &Observation,
    current: CellId,
    map: &SearchMap,
    rng: &mut CpuRng,
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
    choices.sort_unstable_by_key(|cell| (cell.row, cell.column));
    let chosen = if choices.len() > 1 {
        rng.bounded_u32(choices.len() as u32).ok()? as usize
    } else {
        0
    };
    Some(Guidance {
        first_step: *choices.get(chosen)?,
        goal: None,
    })
}

impl SearchMap {
    pub(super) fn value(&self, observation: &Observation, cell: CellId) -> u8 {
        cell_index(observation, cell).map_or(0, |index| self.values[index])
    }
    fn add_goal(&mut self, index: usize, value: u8) {
        self.values[index] = self.values[index].max(value);
        self.highest_value = self.highest_value.max(value);
    }

    pub(super) fn transitable(&self, observation: &Observation, cell: CellId) -> bool {
        let Some(index) = cell_index(observation, cell) else {
            return false;
        };
        observation.parts().arena.terrain.get(index) == Some(&Terrain::Floor)
            && (crate::cpu::capabilities::can_pass_known_soft_blocks(observation)
                || !self.blocked[index])
            && !self.forbidden[index]
    }
}

pub(super) fn pickup_value(
    observation: &Observation,
    actor: &VisibleActor,
    kind: PickupKind,
) -> Option<u8> {
    let rules = crate::cpu::rules_config(observation)?;
    if let Some((value, _)) = supported_weapon(actor, kind) {
        return value;
    }
    if let Some((value, _)) = equipment_pickup(actor, kind) {
        return value;
    }
    match kind {
        PickupKind::BombCapacity if actor.bomb_capacity < rules.bomb_capacity_cap() => Some(7),
        PickupKind::BlastRange if actor.bomb_range < rules.bomb_range_cap() => Some(6),
        PickupKind::MovementSpeed if actor.speed != SpeedLevel::Four => Some(6),
        PickupKind::Heart if actor.equipment != Some(Equipment::Heart) => Some(9),
        PickupKind::Shield => useful_effect(actor, EffectKind::Shield).then_some(8),
        PickupKind::Stopwatch => useful_effect(actor, EffectKind::Stopwatch).then_some(5),
        PickupKind::GoldenFlame if actor.bomb_range < GOLDEN_FLAME_RANGE => {
            useful_effect(actor, EffectKind::GoldenFlame).then_some(6)
        }
        PickupKind::ShortFuse => useful_effect(actor, EffectKind::ShortFuse).then_some(6),
        PickupKind::InvisibilityCloak => {
            useful_effect(actor, EffectKind::InvisibilityCloak).then_some(5)
        }
        PickupKind::Kick
        | PickupKind::Punch
        | PickupKind::Bracelet
        | PickupKind::WallPass
        | PickupKind::RemoteBomb
        | PickupKind::PiercingBomb
        | PickupKind::PowerBomb
        | PickupKind::Landmine
        | PickupKind::Diarrhea
        | PickupKind::SlowFoot
        | PickupKind::ShortFlame
        | PickupKind::LongFuse
        | PickupKind::ReverseControls
        | PickupKind::InvisibleBombs
        | PickupKind::OilBarrel
        | PickupKind::Teleporter
        | PickupKind::BombCapacity
        | PickupKind::BlastRange
        | PickupKind::MovementSpeed
        | PickupKind::Heart
        | PickupKind::GoldenFlame => None,
    }
}

fn pickup_transitable(actor: &VisibleActor, kind: PickupKind) -> bool {
    if let Some((_, transitable)) = supported_weapon(actor, kind) {
        return transitable;
    }
    if let Some((_, transitable)) = equipment_pickup(actor, kind) {
        return transitable;
    }
    match kind {
        PickupKind::RemoteBomb
        | PickupKind::PiercingBomb
        | PickupKind::PowerBomb
        | PickupKind::Landmine
        | PickupKind::Diarrhea
        | PickupKind::SlowFoot
        | PickupKind::ShortFlame
        | PickupKind::LongFuse
        | PickupKind::ReverseControls
        | PickupKind::InvisibleBombs
        | PickupKind::OilBarrel
        | PickupKind::Teleporter => false,
        PickupKind::Kick | PickupKind::Punch | PickupKind::Bracelet | PickupKind::WallPass => {
            actor.equipment != Some(Equipment::Heart)
        }
        PickupKind::BombCapacity
        | PickupKind::BlastRange
        | PickupKind::MovementSpeed
        | PickupKind::Heart
        | PickupKind::Shield
        | PickupKind::Stopwatch
        | PickupKind::InvisibilityCloak
        | PickupKind::GoldenFlame
        | PickupKind::ShortFuse => true,
    }
}

fn useful_effect(actor: &VisibleActor, kind: EffectKind) -> bool {
    actor
        .effects
        .iter()
        .find(|effect| effect.kind == kind)
        .is_none_or(|effect| u64::from(effect.remaining_ticks) <= kind.duration_ticks() / 2)
}
