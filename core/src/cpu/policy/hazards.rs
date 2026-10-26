use crate::cpu::{BombKnowledge, Observation, ObservedActor};
use crate::game::{CellId, EffectKind, Terrain};

pub(super) fn passable(observation: &Observation, cell: CellId) -> bool {
    let arena = &observation.parts().arena;
    if cell.column >= arena.width || cell.row >= arena.height {
        return false;
    }
    let index = usize::from(cell.row) * usize::from(arena.width) + usize::from(cell.column);
    arena.terrain.get(index) == Some(&Terrain::Floor) && !arena.blocks.contains(&cell)
}

pub(super) fn immediately_safe(observation: &Observation, cell: CellId) -> bool {
    escape_step_clear(observation, cell) && !in_visible_blast(observation, cell)
}

pub(super) fn escape_step_clear(observation: &Observation, cell: CellId) -> bool {
    passable(observation, cell)
        && !observation
            .parts()
            .flames
            .iter()
            .any(|flame| flame.cell == cell)
        && !observation.parts().oil.iter().any(|oil| oil.cell == cell)
}

fn in_visible_blast(observation: &Observation, cell: CellId) -> bool {
    let BombKnowledge::Visible(bombs) = &observation.parts().bombs else {
        return false;
    };
    bombs.iter().any(|bomb| {
        if bomb.cell == cell {
            return true;
        }
        let aligned = bomb.cell.row == cell.row || bomb.cell.column == cell.column;
        aligned && manhattan(bomb.cell, cell) <= bomb.range
    })
}

pub(super) fn planting_supported(observation: &Observation) -> bool {
    let parts = observation.parts();
    let BombKnowledge::Visible(bombs) = &parts.bombs else {
        return false;
    };
    let Some(actor) = parts.actors.iter().find_map(|actor| match actor {
        ObservedActor::Visible(actor) if actor.seat == parts.recipient => Some(actor),
        _ => None,
    }) else {
        return false;
    };
    bombs.is_empty()
        && parts.flames.is_empty()
        && parts.oil.is_empty()
        && actor.weapon.is_none()
        && actor.alive
        && !actor.voluntary_actions_frozen
        && actor.live_bomb_count < actor.bomb_capacity
        && !actor.effects.iter().any(|effect| {
            matches!(
                effect.kind,
                EffectKind::GoldenFlame
                    | EffectKind::ShortFuse
                    | EffectKind::Diarrhea
                    | EffectKind::SlowFoot
                    | EffectKind::ShortFlame
                    | EffectKind::LongFuse
                    | EffectKind::ReverseControls
                    | EffectKind::InvisibleBombs
            )
        })
}

pub(super) fn unsupported_control_effect(actor: &crate::cpu::VisibleActor) -> bool {
    actor.effects.iter().any(|effect| {
        matches!(
            effect.kind,
            EffectKind::Diarrhea | EffectKind::SlowFoot | EffectKind::ReverseControls
        )
    })
}

pub(super) const fn manhattan(first: CellId, second: CellId) -> u16 {
    first
        .column
        .abs_diff(second.column)
        .saturating_add(first.row.abs_diff(second.row))
}
