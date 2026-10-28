use std::collections::{BTreeMap, BTreeSet};

use crate::game::arena::{Arena, Terrain};
use crate::game::enabled_items::EnabledItems;
use crate::game::oil::MAX_OIL_BARRELS;
use crate::game::pickups::PickupKind;
use crate::game::types::{CellId, PlayerCount};
use crate::room::BoardSize;

use super::generated_v3::{ArenaGenerationConfig, GenerationError};
use super::random_words::{ChaChaWords, RandomWords};
use super::{GeneratedArena, GenerationRecord, sample_percent, sample_pickup};

pub const VARIABLE_GENERATOR_VERSION: u16 = 4;

pub fn generate_variable_multiplayer_arena(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
) -> Result<GeneratedArena, GenerationError> {
    Ok(generate_with_source(
        seed,
        enabled,
        config,
        &mut ChaChaWords::seeded(seed),
    ))
}

fn generate_with_source(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
    source: &mut impl RandomWords,
) -> GeneratedArena {
    let spawns = round_spawns(
        config.board,
        config.player_count,
        config.assignment.round_number().get(),
    );
    let terrain = board_terrain(config.board);
    let generated =
        generate_layers(source, enabled, config.board).and_then(|(blocks, concealed)| {
            let arena =
                Arena::new_for_board(config.board, terrain.clone(), blocks, spawns.clone()).ok()?;
            validate_reflections(&arena, &concealed).then_some((arena, concealed))
        });
    let (arena, concealed_pickups, fallback_used) = match generated {
        Some((arena, concealed)) => (arena, concealed, false),
        None => (
            fallback_arena(config.board, terrain, spawns),
            BTreeMap::new(),
            true,
        ),
    };
    GeneratedArena {
        arena,
        concealed_pickups,
        record: GenerationRecord {
            version: VARIABLE_GENERATOR_VERSION,
            seed,
            fallback_used,
        },
    }
}

fn generate_layers(
    source: &mut impl RandomWords,
    enabled: EnabledItems,
    board: BoardSize,
) -> Option<(BTreeSet<CellId>, BTreeMap<CellId, PickupKind>)> {
    let mut blocks = BTreeSet::new();
    let mut concealed = BTreeMap::new();
    for orbit in eligible_orbits(board) {
        if sample_percent(source)? >= 65 {
            continue;
        }
        blocks.extend(orbit.iter().copied());
        if sample_percent(source)? >= 30 {
            continue;
        }
        let kind = sample_pickup(source, enabled)?;
        if kind == PickupKind::OilBarrel && !oil_orbit_fits(&concealed, orbit.len()) {
            continue;
        }
        concealed.extend(orbit.into_iter().map(|cell| (cell, kind)));
    }
    Some((blocks, concealed))
}

fn board_terrain(board: BoardSize) -> Vec<Terrain> {
    let (width, height) = board.dimensions();
    let mut terrain = vec![Terrain::Floor; board.cell_count()];
    for row in 0..height {
        for column in 0..width {
            if row == 0
                || column == 0
                || row == height - 1
                || column == width - 1
                || (row % 2 == 0 && column % 2 == 0)
            {
                terrain[index(CellId::new(column, row), width)] = Terrain::Wall;
            }
        }
    }
    terrain
}

fn eligible_orbits(board: BoardSize) -> Vec<Vec<CellId>> {
    let (width, height) = board.dimensions();
    let terrain = board_terrain(board);
    let reserved = reserved_cells(board);
    let mut visited = BTreeSet::new();
    let mut orbits = Vec::new();
    for row in 1..height - 1 {
        for column in 1..width - 1 {
            let cell = CellId::new(column, row);
            if visited.contains(&cell) || reserved.contains(&cell) {
                continue;
            }
            let orbit = reflection_orbit(cell, board);
            visited.extend(orbit.iter().copied());
            if orbit.iter().all(|candidate| {
                terrain[index(*candidate, width)] == Terrain::Floor && !reserved.contains(candidate)
            }) {
                orbits.push(orbit);
            }
        }
    }
    orbits
}

fn reflection_orbit(cell: CellId, board: BoardSize) -> Vec<CellId> {
    let reflected_column = board.width() - 1 - cell.column;
    let reflected_row = board.height() - 1 - cell.row;
    let mut orbit = vec![
        cell,
        CellId::new(reflected_column, cell.row),
        CellId::new(cell.column, reflected_row),
        CellId::new(reflected_column, reflected_row),
    ];
    orbit.sort();
    orbit.dedup();
    orbit
}

fn round_spawns(board: BoardSize, count: PlayerCount, round: u32) -> Vec<CellId> {
    let mut corners = interior_corners(board).to_vec();
    match count {
        PlayerCount::Two => vec![corners[0], corners[3]],
        PlayerCount::Three => {
            corners.remove(((round - 1) as usize + 3) % 4);
            corners
        }
        PlayerCount::Four => corners,
    }
}

fn interior_corners(board: BoardSize) -> [CellId; 4] {
    let right = board.width() - 2;
    let bottom = board.height() - 2;
    [
        CellId::new(1, 1),
        CellId::new(right, 1),
        CellId::new(1, bottom),
        CellId::new(right, bottom),
    ]
}

fn reserved_cells(board: BoardSize) -> BTreeSet<CellId> {
    interior_corners(board)
        .into_iter()
        .flat_map(|spawn| {
            let horizontal = if spawn.column == 1 {
                spawn.column + 1
            } else {
                spawn.column - 1
            };
            let vertical = if spawn.row == 1 {
                spawn.row + 1
            } else {
                spawn.row - 1
            };
            [
                spawn,
                CellId::new(horizontal, spawn.row),
                CellId::new(spawn.column, vertical),
            ]
        })
        .collect()
}

fn validate_reflections(arena: &Arena, concealed: &BTreeMap<CellId, PickupKind>) -> bool {
    for row in 0..arena.height() {
        for column in 0..arena.width() {
            let cell = CellId::new(column, row);
            for reflected in reflection_orbit(cell, arena.board()) {
                if arena.terrain(cell) != arena.terrain(reflected)
                    || arena.blocks().contains(&cell) != arena.blocks().contains(&reflected)
                    || concealed.get(&cell) != concealed.get(&reflected)
                {
                    return false;
                }
            }
        }
    }
    concealed.keys().all(|cell| arena.blocks().contains(cell))
}

fn oil_orbit_fits(concealed: &BTreeMap<CellId, PickupKind>, orbit_size: usize) -> bool {
    concealed
        .values()
        .filter(|kind| **kind == PickupKind::OilBarrel)
        .count()
        + orbit_size
        <= MAX_OIL_BARRELS
}

fn fallback_arena(board: BoardSize, terrain: Vec<Terrain>, spawns: Vec<CellId>) -> Arena {
    Arena::new_for_board(board, terrain, BTreeSet::new(), spawns)
        .expect("the variable empty fallback arena must remain valid")
}

fn index(cell: CellId, width: u16) -> usize {
    usize::from(cell.row) * usize::from(width) + usize::from(cell.column)
}

#[cfg(test)]
#[path = "generated_v4_tests.rs"]
mod tests;
