use std::collections::{BTreeMap, BTreeSet};

use super::projection_facts::WorldFacts;
use super::{BombTrigger, ProjectionError, Terrain};

pub(crate) fn validate(world: &WorldFacts) -> Result<(), ProjectionError> {
    if world.bombs.len() > super::MAX_BOMBS
        || !world
            .bombs
            .windows(2)
            .all(|pair| pair[0].id.get() < pair[1].id.get())
    {
        return Err(ProjectionError::InvalidBombs);
    }
    let actor_ids = world
        .actors
        .iter()
        .map(|actor| actor.view.id)
        .collect::<BTreeSet<_>>();
    let blocks = world
        .arena
        .blocks
        .iter()
        .copied()
        .map(super::projection_world_validation::cell_key)
        .collect::<BTreeSet<_>>();
    let mut cells = BTreeSet::new();
    let mut counts = BTreeMap::new();
    let mut remote_counts = BTreeMap::new();
    for bomb in &world.bombs {
        if actor_ids.contains(&bomb.id)
            || !actor_ids.contains(&bomb.owner)
            || !cells.insert(super::projection_world_validation::cell_key(bomb.cell))
            || blocks.contains(&super::projection_world_validation::cell_key(bomb.cell))
            || super::projection_world_validation::terrain_at(world, bomb.cell)
                != Some(Terrain::Floor)
            || !(1..=8).contains(&bomb.range)
            || bomb.placed_at > world.physical_tick
            || is_due_unpaused(bomb.trigger) && world.result.is_none()
        {
            return Err(ProjectionError::InvalidBombs);
        }
        *counts.entry(bomb.owner).or_insert(0_u8) += 1;
        if bomb.trigger == BombTrigger::remote() {
            *remote_counts.entry(bomb.owner).or_insert(0_u8) += 1;
        }
    }
    for actor in &world.actors {
        let count = counts.get(&actor.view.id).copied().unwrap_or(0);
        if count != actor.view.live_bomb_count
            || count > actor.view.bomb_capacity
            || remote_counts.get(&actor.view.id).copied().unwrap_or(0) > 1
        {
            return Err(ProjectionError::InvalidBombs);
        }
    }
    Ok(())
}

fn is_due_unpaused(trigger: BombTrigger) -> bool {
    matches!(
        trigger.observation(),
        crate::protocol::v7::BombTriggerObservation::Timed {
            remaining_ticks: 0,
            paused: false
        }
    )
}
