use std::collections::BTreeSet;

use crate::game::{CellId, SpikePitClock, SpikePitView, Terrain};
use crate::room::SpikePitConfig;

use super::observation::{ObservationError, ObservationParts, ObservedRespawnLairs};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObservedSpikePits {
    config: SpikePitConfig,
    pits: Vec<SpikePitView>,
}

impl ObservedSpikePits {
    #[must_use]
    pub fn new(config: SpikePitConfig, pits: Vec<SpikePitView>) -> Self {
        Self { config, pits }
    }

    #[must_use]
    pub const fn config(&self) -> SpikePitConfig {
        self.config
    }

    #[must_use]
    pub fn pits(&self) -> &[SpikePitView] {
        &self.pits
    }
}

pub(super) fn validate_spike_pits(
    parts: &ObservationParts,
    respawn_lairs: Option<&ObservedRespawnLairs>,
    observed: &ObservedSpikePits,
) -> Result<(), ObservationError> {
    let invalid = || ObservationError::InvalidActors;
    let SpikePitConfig::Enabled(settings) = observed.config else {
        return Err(invalid());
    };
    if !matches!(observed.pits.len(), 0 | 2 | crate::game::MAX_SPIKE_PITS)
        || observed
            .pits
            .windows(2)
            .any(|pair| pair[0].cell >= pair[1].cell)
        || observed.pits.iter().any(|pit| {
            SpikePitClock::from_observation(pit.phase, pit.remaining_ticks, parts.tick, settings)
                .is_err()
        })
    {
        return Err(invalid());
    }
    if observed.pits.is_empty() {
        return Ok(());
    }

    let cells = observed
        .pits
        .iter()
        .map(|pit| pit.cell)
        .collect::<BTreeSet<_>>();
    let lairs = respawn_lairs
        .into_iter()
        .flat_map(|lairs| lairs.lairs().iter().map(|lair| lair.cell))
        .collect::<BTreeSet<_>>();
    if observed.pits.iter().all(|pit| {
        site_is_publicly_valid(parts, pit.cell, &lairs)
            && reflections_exist(parts, pit.cell, &cells)
    }) {
        Ok(())
    } else {
        Err(invalid())
    }
}

fn site_is_publicly_valid(
    parts: &ObservationParts,
    cell: CellId,
    lairs: &BTreeSet<CellId>,
) -> bool {
    parts
        .arena
        .index(cell)
        .is_some_and(|index| parts.arena.terrain[index] == Terrain::Floor)
        && !parts.arena.blocks.contains(&cell)
        && !parts.oil.iter().any(|oil| oil.cell == cell)
        && !lairs.contains(&cell)
        && outside_spawn_reservations(parts, cell)
}

fn outside_spawn_reservations(parts: &ObservationParts, cell: CellId) -> bool {
    let right = parts.arena.width - 2;
    let bottom = parts.arena.height - 2;
    [
        CellId::new(1, 1),
        CellId::new(right, 1),
        CellId::new(1, bottom),
        CellId::new(right, bottom),
    ]
    .into_iter()
    .all(|spawn| spawn.column.abs_diff(cell.column) + spawn.row.abs_diff(cell.row) > 1)
}

fn reflections_exist(parts: &ObservationParts, cell: CellId, cells: &BTreeSet<CellId>) -> bool {
    let reflected_column = parts.arena.width - 1 - cell.column;
    let reflected_row = parts.arena.height - 1 - cell.row;
    cells.contains(&CellId::new(reflected_column, cell.row))
        && cells.contains(&CellId::new(cell.column, reflected_row))
        && cells.contains(&CellId::new(reflected_column, reflected_row))
}
