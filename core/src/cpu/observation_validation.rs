use std::collections::BTreeSet;

use crate::game::{CellId, EffectKind, SeatId, Tick};
use crate::room::BoardSize;

use super::observation::{
    BombKnowledge, MAX_MULTIPLAYER_OBSERVED_BOMBS, MAX_OBSERVED_BOMBS, MAX_OBSERVED_EFFECTS,
    ObservationError, ObservedActor, ObservedArena, ObservedEffect, ObservedRules,
    UnknownActorFacts, UnknownBombFacts,
};

pub(super) fn validate_clock(tick: Tick, rules: ObservedRules) -> Result<(), ObservationError> {
    if rules.tick_rate == 0
        || u64::from(rules.running_updates) > tick.0
        || rules
            .running_updates
            .checked_add(rules.remaining_ticks)
            .is_none()
    {
        return Err(ObservationError::InvalidClock);
    }
    Ok(())
}

pub(super) fn validate_arena(
    rules_version: u16,
    arena: &ObservedArena,
) -> Result<(), ObservationError> {
    let Some(board) = arena.board() else {
        return Err(ObservationError::InvalidArena);
    };
    if (rules_version < 6 && board != BoardSize::FifteenByThirteen)
        || arena.terrain.len() != board.cell_count()
        || arena.blocks.len() > board.cell_count()
    {
        return Err(ObservationError::InvalidArena);
    }
    validate_cells(arena, &arena.blocks, |cell| *cell).map_err(|()| ObservationError::InvalidArena)
}

pub(super) fn validate_actors(
    rules_version: u16,
    recipient: SeatId,
    arena: &ObservedArena,
    actors: &[ObservedActor],
) -> Result<(), ObservationError> {
    if !(1..=8).contains(&rules_version)
        || (rules_version < 6 && actors.len() != 2)
        || (rules_version >= 6 && !(2..=4).contains(&actors.len()))
        || actors
            .iter()
            .enumerate()
            .any(|(index, actor)| actor.seat().index() != index)
    {
        return Err(ObservationError::InvalidActors);
    }
    let mut seats = BTreeSet::new();
    let mut ids = BTreeSet::new();
    if actors
        .iter()
        .any(|actor| !seats.insert(actor.seat()) || !ids.insert(actor.id()))
    {
        return Err(ObservationError::InvalidActors);
    }
    let Some(actor) = actors.iter().find(|actor| actor.seat() == recipient) else {
        return Err(ObservationError::InvalidActors);
    };
    if !matches!(actor, ObservedActor::Visible(_)) {
        return Err(ObservationError::InvalidActors);
    }
    for actor in actors {
        if let ObservedActor::Visible(actor) = actor {
            validate_effects(&actor.effects)?;
            if actor.unknown != UnknownActorFacts::ALL
                || actor.position.x < 0
                || actor.position.y < 0
                || actor.position.x >= i32::from(arena.width) * crate::game::types::UNITS_PER_CELL
                || actor.position.y >= i32::from(arena.height) * crate::game::types::UNITS_PER_CELL
            {
                return Err(ObservationError::InvalidActors);
            }
        }
    }
    Ok(())
}

fn validate_effects(effects: &[ObservedEffect]) -> Result<(), ObservationError> {
    if effects.len() > MAX_OBSERVED_EFFECTS
        || effects.iter().any(|effect| effect.remaining_ticks == 0)
    {
        return Err(ObservationError::InvalidEffects);
    }
    let mut ordinary = BTreeSet::new();
    let mut slow_foot = 0;
    for effect in effects {
        if effect.kind == EffectKind::SlowFoot {
            slow_foot += 1;
            if slow_foot > crate::game::effects::MAX_SLOW_FOOT_STACKS {
                return Err(ObservationError::InvalidEffects);
            }
        } else if !ordinary.insert(effect.kind) {
            return Err(ObservationError::InvalidEffects);
        }
    }
    Ok(())
}

pub(super) fn validate_bombs(
    tick: Tick,
    rules_version: u16,
    arena: &ObservedArena,
    bombs: &BombKnowledge,
) -> Result<(), ObservationError> {
    let BombKnowledge::Visible(bombs) = bombs else {
        return Ok(());
    };
    let max_bombs = if rules_version >= 6 {
        MAX_MULTIPLAYER_OBSERVED_BOMBS
    } else {
        MAX_OBSERVED_BOMBS
    };
    if bombs.len() > max_bombs {
        return Err(ObservationError::InvalidBombs);
    }
    let mut ids = BTreeSet::new();
    if bombs.iter().any(|bomb| {
        bomb.unknown != UnknownBombFacts::ALL
            || bomb.placed_at > tick
            || !arena.contains(bomb.cell)
            || !ids.insert(bomb.id)
    }) {
        return Err(ObservationError::InvalidBombs);
    }
    Ok(())
}

pub(super) fn validate_cells<T>(
    arena: &ObservedArena,
    values: &[T],
    cell: impl Fn(&T) -> CellId,
) -> Result<(), ()> {
    let mut occupied = BTreeSet::new();
    if values
        .iter()
        .map(cell)
        .any(|cell| !arena.contains(cell) || !occupied.insert(cell))
    {
        return Err(());
    }
    Ok(())
}
