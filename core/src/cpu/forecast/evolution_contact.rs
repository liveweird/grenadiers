use crate::cpu::{BombKnowledge, Observation, ObservedBomb, ObservedBombTrigger};

use super::ForecastEvolution;

impl ForecastEvolution {
    /// Conservatively charges every collection traversal performed by the
    /// narrow contact-bound scene matcher. It supports only an empty initial
    /// bomb/flame/oil scene and at most the certified own bomb.
    pub(crate) fn contact_scene_work(
        &self,
        basis: &Observation,
        current: &Observation,
    ) -> Option<u32> {
        let old = basis.parts();
        let new = current.parts();
        let (BombKnowledge::Visible(old_bombs), BombKnowledge::Visible(new_bombs)) =
            (&old.bombs, &new.bombs)
        else {
            return None;
        };
        if !old_bombs.is_empty()
            || new_bombs.len() > 1
            || !old.flames.is_empty()
            || !old.oil.is_empty()
            || !new.oil.is_empty()
        {
            return None;
        }
        let pickups = old.pickups.len().checked_add(new.pickups.len())?;
        let blocks = old
            .arena
            .blocks
            .len()
            .checked_add(new.arena.blocks.len())?
            .checked_mul(2)?;
        let block_removals = old
            .arena
            .blocks
            .len()
            .checked_mul(self.block_removals.len())?;
        let total = 32_usize
            .checked_add(old.arena.terrain.len())?
            .checked_add(pickups)?
            .checked_add(blocks)?
            .checked_add(block_removals)?
            .checked_add(self.bomb_removals.len().checked_mul(4)?)?
            .checked_add(new_bombs.len().checked_mul(4)?)?
            .checked_add(self.flame_changes.len().checked_mul(4)?)?
            .checked_add(new.flames.len().checked_mul(3)?)?;
        u32::try_from(total).ok()
    }

    pub(crate) fn contact_actor_work(&self) -> Option<u32> {
        let per_bomb = self.bomb_removals.len().checked_add(2)?;
        u32::try_from(self.bomb_owners.len().checked_mul(per_bomb)?).ok()
    }
}

pub(super) fn bomb_matches(before: ObservedBomb, after: ObservedBomb, elapsed: u64) -> bool {
    before.id == after.id
        && before.placed_at == after.placed_at
        && before.cell == after.cell
        && before.owner == after.owner
        && before.range == after.range
        && before.blast == after.blast
        && before.unknown == after.unknown
        && trigger_matches(before.trigger, after.trigger, elapsed)
}

fn trigger_matches(before: ObservedBombTrigger, after: ObservedBombTrigger, elapsed: u64) -> bool {
    match (before, after) {
        (
            ObservedBombTrigger::Timed {
                remaining_ticks: old,
                paused: old_pause,
            },
            ObservedBombTrigger::Timed {
                remaining_ticks: new,
                paused: new_pause,
            },
        ) => {
            old_pause == new_pause
                && if old_pause {
                    old == new
                } else {
                    u64::from(old).checked_sub(elapsed) == Some(u64::from(new))
                }
        }
        (ObservedBombTrigger::Remote, ObservedBombTrigger::Remote)
        | (ObservedBombTrigger::Contact, ObservedBombTrigger::Contact) => true,
        _ => false,
    }
}
