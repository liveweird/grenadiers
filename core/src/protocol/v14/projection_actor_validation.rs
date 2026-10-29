use std::collections::BTreeSet;

use super::projection_facts::WorldFacts;
use super::{ProjectionError, Seat};

pub(crate) fn capture(
    state: &crate::game::State,
    actor: &crate::game::Actor,
    physical_tick: crate::protocol::Tick,
) -> Result<super::projection_facts::ActorFacts, ProjectionError> {
    use super::projection_capture_wire as wire;
    use super::{DamageGraceTicks, EffectView, EffectViews, InventoryView, VisibleActorView};
    use crate::protocol::EntityId;
    let storm_slow_remaining_ticks = match actor.storm_slow_until() {
        Some(until) => u16::try_from(
            until
                .0
                .checked_sub(physical_tick.get())
                .ok_or(ProjectionError::InvalidEffects)?,
        )
        .map_err(|_| ProjectionError::InvalidEffects)?,
        None => 0,
    };
    let effects = actor
        .effect_observations()
        .map(|(kind, expires_at)| {
            let remaining = expires_at
                .0
                .checked_sub(physical_tick.get())
                .ok_or(ProjectionError::InvalidEffects)?;
            EffectView::new(
                wire::effect(kind),
                u16::try_from(remaining).map_err(|_| ProjectionError::InvalidEffects)?,
            )
            .map_err(|_| ProjectionError::InvalidEffects)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let (grace, grace_received_at) = match actor.damage_grace_observation() {
        Some(grace) => {
            physical_tick
                .get()
                .checked_sub(grace.received_at().0)
                .ok_or(ProjectionError::InvalidEffects)?;
            let remaining =
                u8::try_from(grace.remaining_ticks(crate::game::Tick(physical_tick.get())))
                    .map_err(|_| ProjectionError::InvalidEffects)?;
            (remaining, Some(wire::tick(grace.received_at())?))
        }
        None => (0, None),
    };
    let inventory = *actor.inventory();
    let view = VisibleActorView {
        id: EntityId::new(actor.id().0),
        seat: wire::seat(actor.seat()),
        position: wire::position(actor.position())?,
        heading: wire::direction(actor.heading()),
        speed_level: wire::speed(actor.speed()),
        alive: actor.alive(),
        bomb_capacity: u8::try_from(actor.bomb_capacity())
            .map_err(|_| ProjectionError::InvalidActors)?,
        bomb_range: u8::try_from(actor.bomb_range()).map_err(|_| ProjectionError::InvalidActors)?,
        live_bomb_count: u8::try_from(state.live_bomb_count(actor.seat()))
            .map_err(|_| ProjectionError::InvalidActors)?,
        inventory: InventoryView {
            equipment: inventory.equipment().map(wire::equipment),
            weapon: inventory.weapon().map(wire::weapon),
        },
        effects: EffectViews::new(effects).map_err(|_| ProjectionError::InvalidEffects)?,
        damage_grace_remaining_ticks: DamageGraceTicks::new(grace)
            .map_err(|_| ProjectionError::InvalidEffects)?,
        voluntary_actions_frozen: false,
        last_teleport_tick: actor.last_teleport_tick().map(wire::tick).transpose()?,
    };
    Ok(super::projection_facts::ActorFacts {
        view,
        damage_grace_received_at: grace_received_at,
        last_teleport_tick: actor.last_teleport_tick().map(wire::tick).transpose()?,
        last_successful_punch_tick: actor
            .last_successful_punch_tick()
            .map(wire::tick)
            .transpose()?,
        last_successful_shove_tick: actor
            .last_successful_shove_tick()
            .map(wire::tick)
            .transpose()?,
        soft_wall_grace: actor
            .soft_wall_grace_observations()
            .map(wire::cell)
            .collect::<Result<_, _>>()?,
        unlit_oil_grace: actor
            .unlit_oil_grace
            .iter()
            .copied()
            .map(wire::cell)
            .collect::<Result<_, _>>()?,
        storm_slow_remaining_ticks,
    })
}

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
