use crate::game;

use super::projection_facts::{ProjectionError, PunchBombFacts, WorldFacts};
use super::{BombTriggerObservation, BombView, ItemId, ItemSet};

pub(super) fn capture_bomb(bomb: game::Bomb, view: BombView) -> PunchBombFacts {
    facts(view, bomb.has_been_punched())
}

pub(super) fn capture_event_bomb(view: BombView, has_been_punched: bool) -> PunchBombFacts {
    facts(view, has_been_punched)
}

const fn facts(bomb: BombView, has_been_punched: bool) -> PunchBombFacts {
    PunchBombFacts {
        bomb,
        has_been_punched,
    }
}

pub(super) fn validate_world(
    world: &WorldFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    validate_actor_history(world, supported_items)?;
    if world.punch_bombs.len() != world.bombs.len() {
        return Err(ProjectionError::InvalidBombs);
    }
    for bomb in &world.bombs {
        let facts = world
            .punch_bombs
            .get(&bomb.id)
            .copied()
            .ok_or(ProjectionError::InvalidBombs)?;
        validate_bomb(world, supported_items, *bomb, facts, false)
            .map_err(|_| ProjectionError::InvalidBombs)?;
    }
    Ok(())
}

pub(super) fn validate_event(
    world: &WorldFacts,
    bomb: BombView,
    facts: Option<PunchBombFacts>,
    placement: bool,
) -> Result<(), ProjectionError> {
    let facts = facts.ok_or(ProjectionError::InvalidEvent)?;
    validate_bomb(world, world.available_items, bomb, facts, placement)
        .map_err(|_| ProjectionError::InvalidEvent)
}

fn validate_actor_history(
    world: &WorldFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    let round_start = world
        .physical_tick
        .get()
        .checked_sub(u64::from(world.running_updates))
        .ok_or(ProjectionError::InvalidClock)?;
    let punch_allowed = punch_allowed(world, supported_items);
    if world
        .last_successful_punch_ticks
        .iter()
        .flatten()
        .any(|tick| !punch_allowed || tick.get() <= round_start || *tick > world.physical_tick)
    {
        return Err(ProjectionError::InvalidActors);
    }
    Ok(())
}

fn validate_bomb(
    world: &WorldFacts,
    supported_items: ItemSet,
    bomb: BombView,
    facts: PunchBombFacts,
    placement: bool,
) -> Result<(), ()> {
    if facts.bomb != bomb
        || (facts.has_been_punched
            && (placement
                || matches!(bomb.trigger.observation(), BombTriggerObservation::Contact)
                || !punch_allowed(world, supported_items)))
    {
        return Err(());
    }
    Ok(())
}

fn punch_allowed(world: &WorldFacts, supported_items: ItemSet) -> bool {
    supported_items.contains(ItemId::Punch)
        && world.available_items.contains(ItemId::Punch)
        && world.enabled_items.contains(ItemId::Punch)
}
