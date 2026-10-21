use crate::game;

use super::projection_capture_wire::wire_direction;
use super::projection_facts::{BombMotionFacts, KickBombFacts, ProjectionError, WorldFacts};
use super::{BombTriggerObservation, BombView, ItemId, ItemSet};

pub(super) fn capture_bomb(bomb: game::Bomb, view: BombView) -> KickBombFacts {
    facts(view, bomb.motion(), bomb.has_been_kicked())
}

pub(super) fn capture_event_bomb(
    view: BombView,
    motion: Option<game::BombMotion>,
    has_been_kicked: bool,
) -> KickBombFacts {
    facts(view, motion, has_been_kicked)
}

fn facts(bomb: BombView, motion: Option<game::BombMotion>, has_been_kicked: bool) -> KickBombFacts {
    KickBombFacts {
        bomb,
        motion: motion.map(|motion| BombMotionFacts {
            direction: wire_direction(motion.direction()),
            countdown: motion.countdown(),
        }),
        has_been_kicked,
    }
}

pub(super) fn validate_world(
    world: &WorldFacts,
    supported_items: ItemSet,
) -> Result<(), ProjectionError> {
    if world.kick_bombs.len() != world.bombs.len() {
        return Err(ProjectionError::InvalidBombs);
    }
    for bomb in &world.bombs {
        let facts = world
            .kick_bombs
            .get(&bomb.id)
            .copied()
            .ok_or(ProjectionError::InvalidBombs)?;
        validate(world, supported_items, *bomb, facts, false)
            .map_err(|_| ProjectionError::InvalidBombs)?;
    }
    Ok(())
}

pub(super) fn validate_event(
    world: &WorldFacts,
    bomb: BombView,
    facts: Option<KickBombFacts>,
    placement: bool,
) -> Result<(), ProjectionError> {
    let facts = facts.ok_or(ProjectionError::InvalidEvent)?;
    validate(world, world.available_items, bomb, facts, placement)
        .map_err(|_| ProjectionError::InvalidEvent)
}

fn validate(
    world: &WorldFacts,
    supported_items: ItemSet,
    bomb: BombView,
    facts: KickBombFacts,
    placement: bool,
) -> Result<(), ()> {
    let has_motion = facts.motion.is_some();
    let has_history = facts.has_been_kicked;
    let kick_available =
        supported_items.contains(ItemId::Kick) && world.available_items.contains(ItemId::Kick);
    let kick_enabled = world.enabled_items.contains(ItemId::Kick);
    if facts.bomb != bomb
        || facts.motion.is_some_and(|motion| motion.countdown > 9)
        || (has_motion && !has_history)
        || (placement && (has_motion || has_history))
        || (matches!(bomb.trigger.observation(), BombTriggerObservation::Contact)
            && (has_motion || has_history))
        || ((has_motion || has_history) && (!kick_available || !kick_enabled))
    {
        return Err(());
    }
    Ok(())
}
