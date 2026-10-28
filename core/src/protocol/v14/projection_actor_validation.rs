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
            let grace_remaining = u64::from(actor.view.damage_grace_remaining_ticks.get());
            let valid_grace = match actor.damage_grace_received_at {
                None => grace_remaining == 0,
                Some(received) => world
                    .physical_tick
                    .get()
                    .checked_sub(received.get())
                    .is_some_and(|elapsed| 60_u64.saturating_sub(elapsed) == grace_remaining),
            };
            actor.view.seat != seat_at(index)
                || actor.view.speed_level > 4
                || !(1..=8).contains(&actor.view.bomb_capacity)
                || !(1..=8).contains(&actor.view.bomb_range)
                || actor.view.live_bomb_count > actor.view.bomb_capacity
                || super::validate_effects(&actor.view.effects).is_err()
                || !super::projection_geometry::position_in_bounds(world, actor.view.position)
                || actor.view.voluntary_actions_frozen
                || !valid_grace
                || (!actor.view.alive
                    && (actor.view.inventory.equipment.is_some()
                        || actor.view.inventory.weapon.is_some()
                        || !actor.view.effects.as_slice().is_empty()
                        || actor.view.damage_grace_remaining_ticks.get() != 0
                        || actor.damage_grace_received_at.is_some()))
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
