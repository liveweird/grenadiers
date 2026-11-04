use std::collections::BTreeSet;

use super::MAX_SPIKE_PITS;
use crate::game::{Arena, CellId, Terrain};

#[must_use]
pub fn spike_pit_layout_v1(
    arena: &Arena,
    excluded: impl IntoIterator<Item = CellId>,
) -> Vec<CellId> {
    let excluded = excluded.into_iter().collect::<BTreeSet<_>>();
    select_spike_pit_layout(arena, &excluded, |_| true)
}

#[must_use]
pub fn spike_pit_layout_v2(
    arena: &Arena,
    excluded: impl IntoIterator<Item = CellId>,
    active_lairs: impl IntoIterator<Item = CellId>,
) -> Vec<CellId> {
    let excluded = excluded.into_iter().collect::<BTreeSet<_>>();
    let active_lairs = active_lairs.into_iter().collect::<Vec<_>>();
    let rings = crate::game::layout_clearance::lair_floor_ring(arena, active_lairs.iter().copied());
    select_spike_pit_layout(arena, &excluded, |orbit| {
        if orbit.iter().any(|cell| rings.contains(cell)) {
            return false;
        }
        let mut blocked = arena.blocks().clone();
        blocked.extend(excluded.iter().copied());
        blocked.extend(orbit.iter().copied());
        crate::game::layout_clearance::exits_share_corner_network(arena, &blocked, &active_lairs)
    })
}

fn select_spike_pit_layout(
    arena: &Arena,
    excluded: &BTreeSet<CellId>,
    accepts: impl Fn(&[CellId]) -> bool,
) -> Vec<CellId> {
    let center = CellId::new(arena.width() / 2, arena.height() / 2);
    let mut candidates = (1..arena.height() - 1)
        .flat_map(|row| (1..arena.width() - 1).map(move |column| CellId::new(column, row)))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|cell| {
        (
            cell.column.abs_diff(center.column) + cell.row.abs_diff(center.row),
            *cell,
        )
    });

    for target_size in [MAX_SPIKE_PITS, 2] {
        for &candidate in &candidates {
            let orbit = reflected_orbit(arena, candidate);
            if orbit.len() == target_size
                && orbit
                    .iter()
                    .all(|&cell| placement_is_clear(arena, cell, excluded))
                && accepts(&orbit)
            {
                return orbit;
            }
        }
    }
    Vec::new()
}

fn reflected_orbit(arena: &Arena, cell: CellId) -> Vec<CellId> {
    let reflected_column = arena.width() - 1 - cell.column;
    let reflected_row = arena.height() - 1 - cell.row;
    BTreeSet::from([
        cell,
        CellId::new(reflected_column, cell.row),
        CellId::new(cell.column, reflected_row),
        CellId::new(reflected_column, reflected_row),
    ])
    .into_iter()
    .collect()
}

fn placement_is_clear(arena: &Arena, cell: CellId, excluded: &BTreeSet<CellId>) -> bool {
    arena.terrain(cell) == Some(Terrain::Floor)
        && !arena.blocks().contains(&cell)
        && !excluded.contains(&cell)
        && potential_spawns(arena)
            .into_iter()
            .all(|spawn| !spawn_reserved(spawn, cell))
}

fn potential_spawns(arena: &Arena) -> [CellId; 4] {
    let right = arena.width() - 2;
    let bottom = arena.height() - 2;
    [
        CellId::new(1, 1),
        CellId::new(right, 1),
        CellId::new(1, bottom),
        CellId::new(right, bottom),
    ]
}

pub(super) fn spawn_reserved(spawn: CellId, cell: CellId) -> bool {
    spawn.column.abs_diff(cell.column) + spawn.row.abs_diff(cell.row) <= 1
}
