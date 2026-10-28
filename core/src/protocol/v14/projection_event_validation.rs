use crate::game::{EventAudience, PlayerCount, RulesConfig};

use super::projection_facts::{EventFacts, WorldFacts};
use super::{EventKind, ItemId, ProjectionError, Terrain};

pub(crate) fn validate(
    world: &WorldFacts,
    report_tick: crate::protocol::Tick,
    event: &EventFacts,
    rules: &RulesConfig,
) -> Result<(), ProjectionError> {
    let player_count =
        PlayerCount::try_from(world.actors.len()).map_err(|()| ProjectionError::InvalidEvent)?;
    if event.tick != report_tick
        || !event
            .audience
            .is_subset_of(EventAudience::all(player_count))
    {
        return Err(ProjectionError::InvalidEvent);
    }
    match event.kind {
        EventKind::BombPlaced { bomb } => {
            if bomb.placed_at != report_tick {
                return Err(ProjectionError::InvalidEvent);
            }
            validate_event_bomb(world, bomb, event, report_tick, rules, true)
        }
        EventKind::BombDetonated { bomb } => {
            validate_event_bomb(world, bomb, event, report_tick, rules, false)
        }
        EventKind::ActorEliminated { id, seat } => {
            reject_history(event)?;
            world
                .actors
                .iter()
                .any(|actor| actor.view.id == id && actor.view.seat == seat && !actor.view.alive)
                .then_some(())
                .ok_or(ProjectionError::InvalidEvent)
        }
        EventKind::DamagePrevented { id, seat, source } => {
            reject_history(event)?;
            let required = match source {
                super::DamagePreventionSource::Heart => ItemId::Heart,
                super::DamagePreventionSource::Shield => ItemId::Shield,
            };
            (actor_exists(world, id, seat) && world.enabled_items.contains(required))
                .then_some(())
                .ok_or(ProjectionError::InvalidEvent)
        }
        EventKind::PickupCollected {
            id,
            seat,
            cell,
            kind,
        } => {
            reject_history(event)?;
            let item = crate::protocol::item_foundation::projection_items::pickup_item(kind);
            (actor_exists(world, id, seat)
                && super::projection_world_validation::terrain_at(world, cell)
                    == Some(Terrain::Floor)
                && !world.arena.blocks.contains(&cell)
                && item.is_none_or(|value| world.enabled_items.contains(value)))
            .then_some(())
            .ok_or(ProjectionError::InvalidEvent)
        }
        EventKind::RoundEnded { result } => {
            reject_history(event)?;
            (world.result == Some(result))
                .then_some(())
                .ok_or(ProjectionError::InvalidEvent)
        }
    }
}

fn validate_event_bomb(
    world: &WorldFacts,
    bomb: super::BombView,
    event: &EventFacts,
    tick: crate::protocol::Tick,
    rules: &RulesConfig,
    placement: bool,
) -> Result<(), ProjectionError> {
    let history = event.bomb_history.ok_or(ProjectionError::InvalidEvent)?;
    if !(1..=8).contains(&bomb.range)
        || bomb.placed_at > tick
        || super::projection_world_validation::terrain_at(world, bomb.cell) != Some(Terrain::Floor)
        || world.arena.blocks.contains(&bomb.cell)
        || world.actors.iter().any(|actor| actor.view.id == bomb.id)
        || !world.actors.iter().any(|actor| actor.view.id == bomb.owner)
    {
        return Err(ProjectionError::InvalidEvent);
    }
    super::projection_history::validate_bomb(world, bomb, history, tick, rules, placement)
        .map_err(|()| ProjectionError::InvalidEvent)
}

fn reject_history(event: &EventFacts) -> Result<(), ProjectionError> {
    event
        .bomb_history
        .is_none()
        .then_some(())
        .ok_or(ProjectionError::InvalidEvent)
}

fn actor_exists(world: &WorldFacts, id: crate::protocol::EntityId, seat: super::Seat) -> bool {
    world
        .actors
        .iter()
        .any(|actor| actor.view.id == id && actor.view.seat == seat)
}
