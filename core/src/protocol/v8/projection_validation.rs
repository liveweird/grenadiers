use std::collections::{BTreeMap, BTreeSet};

use crate::protocol::{DrawReason, Seat, WireRoundResult};

use super::BombTriggerObservation;
use super::projection_event_validation::{validate_event, validate_timed_bomb};
use super::projection_facts::{
    ProjectionError, ReportFacts, ValidatedReports, ValidatedWorld, WorldFacts,
};
use super::projection_items::{effect_item, equipment_item, pickup_item, push_item, weapon_item};
use super::{
    ARENA_TERRAIN_CELLS, BlastKind, ItemId, MAX_BLOCKS, MAX_BOMBS, MAX_FLAMES, MAX_PICKUPS,
    MAX_RUNNING_UPDATES, Terrain, validate_effects,
};

pub(crate) fn validate_world(facts: WorldFacts) -> Result<ValidatedWorld, ProjectionError> {
    validate_clock(&facts)?;
    validate_arena(&facts)?;
    validate_actors(&facts)?;
    validate_bombs(&facts)?;
    validate_flames(&facts)?;
    validate_pickups(&facts)?;
    validate_result(&facts)?;
    validate_items(&facts)?;
    Ok(ValidatedWorld(facts))
}

pub(crate) fn validate_reports(
    world: &ValidatedWorld,
    reports: Vec<ReportFacts>,
) -> Result<ValidatedReports, ProjectionError> {
    if reports.len() > 3
        || reports
            .windows(2)
            .any(|pair| pair[0].tick.get().checked_add(1) != Some(pair[1].tick.get()))
        || reports
            .iter()
            .any(|report| report.tick > world.0.physical_tick)
        || reports
            .iter()
            .map(|report| report.events.len())
            .sum::<usize>()
            > 96
    {
        return Err(ProjectionError::InvalidReports);
    }
    for report in &reports {
        for event in &report.events {
            validate_event(&world.0, report.tick, event)?;
        }
    }
    Ok(ValidatedReports(reports))
}

fn validate_clock(world: &WorldFacts) -> Result<(), ProjectionError> {
    (world.running_updates <= MAX_RUNNING_UPDATES
        && world.remaining_ticks <= MAX_RUNNING_UPDATES
        && world.running_updates.checked_add(world.remaining_ticks) == Some(MAX_RUNNING_UPDATES))
    .then_some(())
    .ok_or(ProjectionError::InvalidClock)
}

fn validate_arena(world: &WorldFacts) -> Result<(), ProjectionError> {
    let arena = &world.arena;
    if arena.width != 15
        || arena.height != 13
        || arena.terrain.len() != ARENA_TERRAIN_CELLS
        || arena.blocks.len() > MAX_BLOCKS
        || !strict_cells(&arena.blocks)
        || arena
            .blocks
            .iter()
            .any(|&cell| terrain_at(world, cell) != Some(Terrain::Floor))
    {
        return Err(ProjectionError::InvalidArena);
    }
    Ok(())
}

fn validate_actors(world: &WorldFacts) -> Result<(), ProjectionError> {
    let [one, two] = &world.actors;
    if one.seat != Seat::One
        || two.seat != Seat::Two
        || one.id == two.id
        || world.actors.iter().any(|actor| {
            actor.speed_level > 4
                || !(1..=8).contains(&actor.bomb_capacity)
                || !(1..=8).contains(&actor.bomb_range)
                || actor.live_bomb_count > actor.bomb_capacity
                || validate_effects(&actor.effects).is_err()
                || (!actor.alive
                    && (actor.inventory.equipment.is_some()
                        || actor.inventory.weapon.is_some()
                        || !actor.effects.as_slice().is_empty()
                        || actor.damage_grace_remaining_ticks.get() != 0))
        })
    {
        return Err(ProjectionError::InvalidActors);
    }
    Ok(())
}

fn validate_bombs(world: &WorldFacts) -> Result<(), ProjectionError> {
    if world.bombs.len() > MAX_BOMBS
        || !world
            .bombs
            .windows(2)
            .all(|pair| pair[0].id.get() < pair[1].id.get())
    {
        return Err(ProjectionError::InvalidBombs);
    }
    let actor_ids = BTreeSet::from([world.actors[0].id.get(), world.actors[1].id.get()]);
    let blocks = cell_set(&world.arena.blocks);
    let mut cells = BTreeSet::new();
    let mut counts = BTreeMap::new();
    let mut timed_count = 0;
    let paused_now = world.actors.iter().any(|actor| {
        actor.alive
            && actor
                .effects
                .as_slice()
                .iter()
                .any(|effect| effect.kind() == super::EffectKind::Stopwatch)
    });
    for bomb in &world.bombs {
        if actor_ids.contains(&bomb.id.get())
            || !actor_ids.contains(&bomb.owner.get())
            || !cells.insert(cell_key(bomb.cell))
            || blocks.contains(&cell_key(bomb.cell))
            || terrain_at(world, bomb.cell) != Some(Terrain::Floor)
            || !(1..=8).contains(&bomb.range)
            || bomb.placed_at > world.physical_tick
            || matches!(
                bomb.trigger.observation(),
                BombTriggerObservation::Timed {
                    remaining_ticks: 0,
                    ..
                }
            ) && world.result.is_none()
        {
            return Err(ProjectionError::InvalidBombs);
        }
        match bomb.trigger.observation() {
            BombTriggerObservation::Timed {
                remaining_ticks, ..
            } => {
                timed_count += 1;
                let Some(timing) = world.timed_bombs.get(&bomb.id).copied() else {
                    return Err(ProjectionError::InvalidBombs);
                };
                validate_timed_bomb(
                    world,
                    *bomb,
                    timing,
                    world.physical_tick,
                    u64::from(remaining_ticks),
                    false,
                )
                .map_err(|_| ProjectionError::InvalidBombs)?;
                if timing.paused != paused_now {
                    return Err(ProjectionError::InvalidBombs);
                }
            }
            BombTriggerObservation::Remote | BombTriggerObservation::Contact => {
                if world.timed_bombs.contains_key(&bomb.id) {
                    return Err(ProjectionError::InvalidBombs);
                }
            }
        }
        *counts.entry(bomb.owner.get()).or_insert(0_u8) += 1;
    }
    if world.timed_bombs.len() != timed_count {
        return Err(ProjectionError::InvalidBombs);
    }
    for actor in &world.actors {
        let count = counts.get(&actor.id.get()).copied().unwrap_or(0);
        if count != actor.live_bomb_count || count > actor.bomb_capacity {
            return Err(ProjectionError::InvalidBombs);
        }
    }
    Ok(())
}

fn validate_flames(world: &WorldFacts) -> Result<(), ProjectionError> {
    if world.flames.len() > MAX_FLAMES
        || !world
            .flames
            .windows(2)
            .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
        || world.flames.iter().any(|flame| {
            terrain_at(world, flame.cell) != Some(Terrain::Floor)
                || world.result.is_none() && flame.expires_at <= world.physical_tick
        })
    {
        return Err(ProjectionError::InvalidFlames);
    }
    Ok(())
}

fn validate_pickups(world: &WorldFacts) -> Result<(), ProjectionError> {
    let blocks = cell_set(&world.arena.blocks);
    let flames = world
        .flames
        .iter()
        .map(|flame| cell_key(flame.cell))
        .collect::<BTreeSet<_>>();
    if world.pickups.len() > MAX_PICKUPS
        || !world
            .pickups
            .windows(2)
            .all(|pair| cell_key(pair[0].cell) < cell_key(pair[1].cell))
        || world.pickups.iter().any(|pickup| {
            terrain_at(world, pickup.cell) != Some(Terrain::Floor)
                || blocks.contains(&cell_key(pickup.cell))
                || flames.contains(&cell_key(pickup.cell))
        })
    {
        return Err(ProjectionError::InvalidPickups);
    }
    Ok(())
}

fn validate_result(world: &WorldFacts) -> Result<(), ProjectionError> {
    let alive = [world.actors[0].alive, world.actors[1].alive];
    let valid = match (world.result, alive) {
        (None, [true, true]) => world.running_updates < MAX_RUNNING_UPDATES,
        (Some(WireRoundResult::Winner { seat: Seat::One }), [true, false])
        | (Some(WireRoundResult::Winner { seat: Seat::Two }), [false, true])
        | (
            Some(WireRoundResult::Draw {
                reason: DrawReason::MutualElimination,
            }),
            [false, false],
        ) => true,
        (
            Some(WireRoundResult::Draw {
                reason: DrawReason::Timeout,
            }),
            [true, true],
        ) => world.running_updates == MAX_RUNNING_UPDATES,
        _ => false,
    };
    valid.then_some(()).ok_or(ProjectionError::InvalidResult)
}

fn validate_items(world: &WorldFacts) -> Result<(), ProjectionError> {
    if !world.enabled_items.is_subset_of(world.available_items) {
        return Err(ProjectionError::InvalidItems);
    }
    let mut required = Vec::new();
    for actor in &world.actors {
        if let Some(item) = actor.inventory.equipment {
            push_item(&mut required, equipment_item(item));
        }
        if let Some(item) = actor.inventory.weapon {
            push_item(&mut required, weapon_item(item));
        }
        for effect in actor.effects.as_slice() {
            push_item(&mut required, effect_item(effect.kind()));
        }
        if actor.damage_grace_remaining_ticks.get() > 0 {
            push_item(&mut required, ItemId::Heart);
        }
    }
    for bomb in &world.bombs {
        match bomb.trigger.observation() {
            BombTriggerObservation::Remote => push_item(&mut required, ItemId::RemoteBomb),
            BombTriggerObservation::Contact => push_item(&mut required, ItemId::Landmine),
            BombTriggerObservation::Timed { paused: true, .. } => {
                push_item(&mut required, ItemId::Stopwatch);
            }
            BombTriggerObservation::Timed { paused: false, .. } => {}
        }
        match bomb.blast {
            BlastKind::Piercing => push_item(&mut required, ItemId::PiercingBomb),
            BlastKind::Power => push_item(&mut required, ItemId::PowerBomb),
            BlastKind::Standard => {}
        }
    }
    for pickup in &world.pickups {
        if let Some(item) = pickup_item(pickup.kind) {
            push_item(&mut required, item);
        }
    }
    if required
        .into_iter()
        .all(|item| world.enabled_items.contains(item))
    {
        Ok(())
    } else {
        Err(ProjectionError::InvalidItems)
    }
}

pub(super) fn terrain_at(world: &WorldFacts, cell: crate::protocol::Cell) -> Option<Terrain> {
    world
        .arena
        .terrain
        .get(usize::from(cell.row()) * 15 + usize::from(cell.column()))
        .copied()
}
fn cell_key(cell: crate::protocol::Cell) -> (u8, u8) {
    (cell.row(), cell.column())
}
fn cell_set(cells: &[crate::protocol::Cell]) -> BTreeSet<(u8, u8)> {
    cells.iter().copied().map(cell_key).collect()
}
fn strict_cells(cells: &[crate::protocol::Cell]) -> bool {
    cells.windows(2).all(|p| cell_key(p[0]) < cell_key(p[1]))
}
