use std::collections::BTreeSet;

use super::projection_facts::WorldFacts;
use super::{ProjectionError, Seat};

pub(crate) fn validate(world: &WorldFacts) -> Result<(), ProjectionError> {
    let count = world.actors.len();
    if !(2..=4).contains(&count) {
        return Err(ProjectionError::InvalidActors);
    }
    let ids = world
        .actors
        .iter()
        .map(|actor| actor.view.id)
        .collect::<BTreeSet<_>>();
    if ids.len() != count
        || world.actors.iter().enumerate().any(|(index, actor)| {
            actor.view.seat != seat_at(index)
                || actor.view.speed_level > 4
                || !(1..=8).contains(&actor.view.bomb_capacity)
                || !(1..=8).contains(&actor.view.bomb_range)
                || actor.view.live_bomb_count > actor.view.bomb_capacity
                || super::validate_effects(&actor.view.effects).is_err()
                || actor.view.position.x() >= u16::from(world.arena.width) * 1_024
                || actor.view.position.y() >= u16::from(world.arena.height) * 1_024
                || actor.view.voluntary_actions_frozen
                || (!actor.view.alive
                    && (actor.view.inventory.equipment.is_some()
                        || actor.view.inventory.weapon.is_some()
                        || !actor.view.effects.as_slice().is_empty()
                        || actor.view.damage_grace_remaining_ticks.get() != 0))
        })
    {
        return Err(ProjectionError::InvalidActors);
    }
    Ok(())
}

const fn seat_at(index: usize) -> Seat {
    match index {
        0 => Seat::One,
        1 => Seat::Two,
        2 => Seat::Three,
        3 => Seat::Four,
        _ => unreachable!(),
    }
}
