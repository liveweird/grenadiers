use std::collections::BTreeSet;

use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{LairView, MAX_LAIRS, MAX_MOBS, MobView, Terrain, lair_layout_v1};
use crate::room::LairConfig;

use super::observation::{BombKnowledge, ObservationError, ObservationParts};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedLairs {
    config: LairConfig,
    lairs: Vec<LairView>,
    mobs: Vec<MobView>,
}

impl ObservedLairs {
    #[must_use]
    pub fn new(config: LairConfig, lairs: Vec<LairView>, mobs: Vec<MobView>) -> Self {
        Self {
            config,
            lairs,
            mobs,
        }
    }

    #[must_use]
    pub const fn config(&self) -> LairConfig {
        self.config
    }

    #[must_use]
    pub fn lairs(&self) -> &[LairView] {
        &self.lairs
    }

    #[must_use]
    pub fn mobs(&self) -> &[MobView] {
        &self.mobs
    }
}

pub(super) fn validate_lairs(
    parts: &ObservationParts,
    observed: &ObservedLairs,
) -> Result<(), ObservationError> {
    let invalid = || ObservationError::InvalidActors;
    let LairConfig::Enabled(settings) = observed.config else {
        return if observed.lairs.is_empty() && observed.mobs.is_empty() {
            Ok(())
        } else {
            Err(invalid())
        };
    };
    let board = parts.arena.board().ok_or_else(invalid)?;
    let expected = lair_layout_v1(board);
    let latest_spawn = parts
        .tick
        .0
        .checked_add(settings.spawn_ticks())
        .ok_or_else(invalid)?;
    if observed.lairs.len() != MAX_LAIRS
        || observed.mobs.len() > MAX_MOBS
        || observed.lairs.iter().zip(expected).any(|(lair, cell)| {
            lair.cell != cell
                || parts
                    .arena
                    .terrain
                    .get(parts.arena.index(cell).unwrap_or(usize::MAX))
                    != Some(&Terrain::Wall)
                || lair.next_spawn_at < parts.tick
                || lair.next_spawn_at.0 > latest_spawn
                || lair.living_mobs > settings.mobs_per_lair()
        })
    {
        return Err(invalid());
    }

    let mut ids = parts
        .actors
        .iter()
        .map(|actor| actor.id())
        .collect::<BTreeSet<_>>();
    if let BombKnowledge::Visible(bombs) = &parts.bombs
        && bombs.iter().any(|bomb| !ids.insert(bomb.id))
    {
        return Err(invalid());
    }
    let width = i32::from(parts.arena.width) * UNITS_PER_CELL;
    let height = i32::from(parts.arena.height) * UNITS_PER_CELL;
    let mut counts = [0_u8; MAX_LAIRS];
    for mob in &observed.mobs {
        let Some(count) = counts.get_mut(usize::from(mob.lair)) else {
            return Err(invalid());
        };
        *count = count.checked_add(1).ok_or_else(invalid)?;
        if !ids.insert(mob.id)
            || mob.position.x < HALF_FOOTPRINT
            || mob.position.y < HALF_FOOTPRINT
            || mob.position.x > width - HALF_FOOTPRINT
            || mob.position.y > height - HALF_FOOTPRINT
        {
            return Err(invalid());
        }
    }
    if observed
        .lairs
        .iter()
        .zip(counts)
        .any(|(lair, count)| lair.living_mobs != count)
    {
        return Err(invalid());
    }
    Ok(())
}
