use crate::cpu::{Observation, WorkBudget};

use super::{PlantContactProof, SourceKind};

impl PlantContactProof {
    pub(crate) fn observation_fits(&self, current: &Observation, work: &mut WorkBudget) -> bool {
        let parts = current.parts();
        if parts.world != self.key.world
            || parts.recipient != self.key.cpu_seat
            || parts.tick < self.basis_tick
            || parts.tick >= self.safe_until
        {
            return false;
        }
        let Some(block_scan) = self
            .retained_blocks
            .len()
            .checked_add(parts.arena.blocks.len())
            .and_then(|count| u32::try_from(count).ok())
        else {
            return false;
        };
        if work.try_take(block_scan).is_err()
            || !self
                .retained_blocks
                .is_subset(&parts.arena.blocks.iter().copied().collect())
        {
            return false;
        }
        let Some(lairs) = current.respawn_lairs() else {
            return false;
        };
        let Some(source_scan) = lairs
            .mobs()
            .len()
            .checked_mul(self.sources.len())
            .and_then(|count| count.checked_mul(2))
            .and_then(|count| u32::try_from(count).ok())
        else {
            return false;
        };
        let Ok(actor_scan) = u32::try_from(parts.actors.len()) else {
            return false;
        };
        let Some(renewal_scan) = source_scan.checked_add(actor_scan) else {
            return false;
        };
        if work.try_take(renewal_scan).is_err()
            || lairs.mobs().iter().any(|mob| {
                let Some(cell) = mob.position.cell() else {
                    return true;
                };
                let Some(component) = self.components.get(&cell) else {
                    return true;
                };
                let existing = self.sources.iter().find(|source| {
                    matches!(source.source.kind, SourceKind::Existing { id, .. } if id == mob.id)
                });
                !existing.map_or_else(
                    || {
                        self.sources.iter().any(|source| {
                            source.component == *component
                                && matches!(
                                    source.source.kind,
                                    SourceKind::PossibleSpawn { lair } if lair == mob.lair
                                )
                        })
                    },
                    |source| {
                        source.component == *component
                            && matches!(
                                source.source.kind,
                                SourceKind::Existing { lair, .. } if lair == mob.lair
                            )
                    },
                )
            })
        {
            return false;
        }
        matches!(
            self.envelope.contains(current, self.safe_until, work),
            Ok(true)
        )
    }
}
