use std::collections::BTreeSet;

use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{LairStatusView, MAX_LAIRS, MAX_MOBS, MobView, Terrain, lair_layout_v1};
use crate::room::{RespawnLairConfig, RespawnLairSettings};

use super::observation::{BombKnowledge, ObservationError, ObservationParts};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedRespawnLairs {
    config: RespawnLairConfig,
    lairs: Vec<LairStatusView>,
    mobs: Vec<MobView>,
}

impl ObservedRespawnLairs {
    #[must_use]
    pub fn new(config: RespawnLairConfig, lairs: Vec<LairStatusView>, mobs: Vec<MobView>) -> Self {
        Self {
            config,
            lairs,
            mobs,
        }
    }

    #[must_use]
    pub const fn config(&self) -> RespawnLairConfig {
        self.config
    }

    #[must_use]
    pub fn lairs(&self) -> &[LairStatusView] {
        &self.lairs
    }

    #[must_use]
    pub fn mobs(&self) -> &[MobView] {
        &self.mobs
    }
}

pub(super) fn validate_respawn_lairs(
    parts: &ObservationParts,
    observed: &ObservedRespawnLairs,
) -> Result<(), ObservationError> {
    let invalid = || ObservationError::InvalidActors;
    let RespawnLairConfig::Enabled(settings) = observed.config else {
        return Err(invalid());
    };
    validate_shape(parts, observed, settings)
        .then_some(())
        .ok_or_else(invalid)
}

fn validate_shape(
    parts: &ObservationParts,
    observed: &ObservedRespawnLairs,
    settings: RespawnLairSettings,
) -> bool {
    let Some(board) = parts.arena.board() else {
        return false;
    };
    let expected = lair_layout_v1(board);
    let latest = parts.tick.0.checked_add(settings.spawn_ticks());
    if observed.lairs.len() != MAX_LAIRS || observed.mobs.len() > MAX_MOBS || latest.is_none() {
        return false;
    }
    let latest = latest.unwrap_or(parts.tick.0);
    if observed.lairs.iter().zip(expected).any(|(lair, cell)| {
        lair.cell != cell
            || parts
                .arena
                .terrain
                .get(parts.arena.index(cell).unwrap_or(usize::MAX))
                != Some(&Terrain::Wall)
            || lair.living_mobs > 1
            || match (lair.living_mobs, lair.next_spawn_at) {
                (1, None) => false,
                (0, Some(at)) => at < parts.tick || at.0 > latest,
                _ => true,
            }
    }) {
        return false;
    }
    let mut ids = parts
        .actors
        .iter()
        .map(|actor| actor.id())
        .collect::<BTreeSet<_>>();
    if let BombKnowledge::Visible(bombs) = &parts.bombs
        && bombs.iter().any(|bomb| !ids.insert(bomb.id))
    {
        return false;
    }
    let width = i32::from(parts.arena.width) * UNITS_PER_CELL;
    let height = i32::from(parts.arena.height) * UNITS_PER_CELL;
    let mut counts = [0_u8; MAX_LAIRS];
    for mob in &observed.mobs {
        let Some(count) = counts.get_mut(usize::from(mob.lair)) else {
            return false;
        };
        *count = count.saturating_add(1);
        if *count > 1
            || !ids.insert(mob.id)
            || mob.position.x < HALF_FOOTPRINT
            || mob.position.y < HALF_FOOTPRINT
            || mob.position.x > width - HALF_FOOTPRINT
            || mob.position.y > height - HALF_FOOTPRINT
        {
            return false;
        }
    }
    observed
        .lairs
        .iter()
        .zip(counts)
        .all(|(lair, count)| lair.living_mobs == count)
}
