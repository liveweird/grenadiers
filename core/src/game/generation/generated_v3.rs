use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::room::BoardSize;

use super::random_words::{ChaChaWords, RandomWords};
use super::{GeneratedArena, GenerationRecord, sample_percent, sample_pickup};
use crate::game::arena::{ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain};
use crate::game::assignment::RoundAssignment;
use crate::game::enabled_items::EnabledItems;
use crate::game::fixtures::classic_terrain;
use crate::game::oil::MAX_OIL_BARRELS;
use crate::game::pickups::PickupKind;
use crate::game::types::{CellId, PlayerCount};

pub const MULTIPLAYER_GENERATOR_VERSION: u16 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArenaGenerationConfig {
    pub player_count: PlayerCount,
    pub board: BoardSize,
    pub assignment: RoundAssignment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenerationError {
    UnsupportedBoard(BoardSize),
}

impl fmt::Display for GenerationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for GenerationError {}

pub fn generate_multiplayer_arena(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
) -> Result<GeneratedArena, GenerationError> {
    if config.board != BoardSize::FifteenByThirteen {
        return Err(GenerationError::UnsupportedBoard(config.board));
    }
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
    let spawns = round_spawns(config.player_count, config.assignment);
    let generated = generate_layers(source, enabled).and_then(|(blocks, concealed)| {
        let arena = Arena::new(classic_terrain(), blocks, spawns.clone()).ok()?;
        validate_reflections(&arena, &concealed).then_some((arena, concealed))
    });
    let (arena, concealed_pickups, fallback_used) = match generated {
        Some((arena, concealed)) => (arena, concealed, false),
        None => (fallback_arena(spawns), BTreeMap::new(), true),
    };
    GeneratedArena {
        arena,
        concealed_pickups,
        record: GenerationRecord {
            version: MULTIPLAYER_GENERATOR_VERSION,
            seed,
            fallback_used,
        },
    }
}

fn generate_layers(
    source: &mut impl RandomWords,
    enabled: EnabledItems,
) -> Option<(BTreeSet<CellId>, BTreeMap<CellId, PickupKind>)> {
    let mut blocks = BTreeSet::new();
    let mut concealed = BTreeMap::new();
    for orbit in eligible_orbits() {
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

fn eligible_orbits() -> Vec<Vec<CellId>> {
    let terrain = classic_terrain();
    let mut visited = BTreeSet::new();
    let mut orbits = Vec::new();
    for row in 1..ARENA_HEIGHT - 1 {
        for column in 1..ARENA_WIDTH - 1 {
            let cell = CellId::new(column, row);
            if visited.contains(&cell) || RESERVED_CELLS.contains(&cell) {
                continue;
            }
            let orbit = reflection_orbit(cell);
            visited.extend(orbit.iter().copied());
            if orbit.iter().all(|candidate| {
                terrain[index(*candidate)] == Terrain::Floor && !RESERVED_CELLS.contains(candidate)
            }) {
                orbits.push(orbit);
            }
        }
    }
    orbits
}

fn reflection_orbit(cell: CellId) -> Vec<CellId> {
    let reflected_column = ARENA_WIDTH - 1 - cell.column;
    let reflected_row = ARENA_HEIGHT - 1 - cell.row;
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

fn round_spawns(count: PlayerCount, assignment: RoundAssignment) -> Vec<CellId> {
    let mut corners = CORNERS.to_vec();
    match count {
        PlayerCount::Two => vec![CORNERS[0], CORNERS[3]],
        PlayerCount::Three => {
            let omitted = ((assignment.round_number().get() - 1) as usize + 3) % 4;
            corners.remove(omitted);
            corners
        }
        PlayerCount::Four => corners,
    }
}

fn validate_reflections(arena: &Arena, concealed: &BTreeMap<CellId, PickupKind>) -> bool {
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            let cell = CellId::new(column, row);
            for reflected in reflection_orbit(cell) {
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
    let existing = concealed
        .values()
        .filter(|kind| **kind == PickupKind::OilBarrel)
        .count();
    existing + orbit_size <= MAX_OIL_BARRELS
}

fn fallback_arena(spawns: Vec<CellId>) -> Arena {
    Arena::new(classic_terrain(), BTreeSet::new(), spawns)
        .expect("the multiplayer empty fallback arena must remain valid")
}

const fn index(cell: CellId) -> usize {
    cell.row as usize * ARENA_WIDTH as usize + cell.column as usize
}

const CORNERS: [CellId; 4] = [
    CellId::new(1, 1),
    CellId::new(13, 1),
    CellId::new(1, 11),
    CellId::new(13, 11),
];

const RESERVED_CELLS: [CellId; 12] = [
    CellId::new(1, 1),
    CellId::new(2, 1),
    CellId::new(1, 2),
    CellId::new(13, 1),
    CellId::new(12, 1),
    CellId::new(13, 2),
    CellId::new(1, 11),
    CellId::new(2, 11),
    CellId::new(1, 10),
    CellId::new(13, 11),
    CellId::new(12, 11),
    CellId::new(13, 10),
];

#[cfg(test)]
#[path = "generated_v3_tests.rs"]
mod tests;
