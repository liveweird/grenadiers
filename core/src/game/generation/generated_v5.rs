use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};

use crate::game::layout_clearance::{corner_openings, floor_neighbors, lair_floor_ring};
use crate::game::{CellId, EnabledItems, lair_layout_v1};
use crate::room::BoardSize;

use super::{
    ArenaConstructionProfile, ArenaGenerationConfig, GeneratedArena, GenerationError,
    generate_multiplayer_arena, generate_variable_multiplayer_arena,
};

pub const LAIR_CLEARANCE_GENERATOR_VERSION: u16 = 5;

pub fn generate_profiled_multiplayer_arena(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
    profile: ArenaConstructionProfile,
    lairs_enabled: bool,
) -> Result<GeneratedArena, GenerationError> {
    let mut generated = base_generation(seed, enabled, config)?;
    if profile == ArenaConstructionProfile::LairClearanceV5 {
        if lairs_enabled {
            apply_lair_clearance(&mut generated);
        }
        generated.record.version = LAIR_CLEARANCE_GENERATOR_VERSION;
    }
    Ok(generated)
}

fn base_generation(
    seed: [u8; 32],
    enabled: EnabledItems,
    config: ArenaGenerationConfig,
) -> Result<GeneratedArena, GenerationError> {
    match config.board {
        BoardSize::FifteenByThirteen => generate_multiplayer_arena(seed, enabled, config),
        BoardSize::ElevenByNine | BoardSize::ThirteenByEleven => {
            generate_variable_multiplayer_arena(seed, enabled, config)
        }
    }
}

fn apply_lair_clearance(generated: &mut GeneratedArena) {
    let arena = &generated.arena;
    let lairs = lair_layout_v1(arena.board());
    let ring = lair_floor_ring(arena, lairs);
    let connector = minimal_connector(arena, &ring);
    let mut reserved = ring;
    for cell in connector {
        reserved.extend(reflection_orbit(arena.board(), cell));
    }
    generated.arena.remove_blocks(&reserved);
    generated
        .concealed_pickups
        .retain(|cell, _| !reserved.contains(cell));
}

fn minimal_connector(arena: &crate::game::Arena, sources: &BTreeSet<CellId>) -> Vec<CellId> {
    let targets = corner_openings(arena).into_iter().collect::<BTreeSet<_>>();
    let mut frontier = BinaryHeap::new();
    let mut costs = BTreeMap::new();
    let mut previous = BTreeMap::new();
    for &source in sources {
        frontier.push(Reverse((0_u16, 0_u16, source)));
        costs.insert(source, (0_u16, 0_u16));
    }
    let target = loop {
        let Some(Reverse((blocks, length, cell))) = frontier.pop() else {
            return Vec::new();
        };
        if costs.get(&cell) != Some(&(blocks, length)) {
            continue;
        }
        if targets.contains(&cell) {
            break cell;
        }
        for neighbor in floor_neighbors(arena, cell) {
            let next = (
                blocks + u16::from(arena.blocks().contains(&neighbor)),
                length + 1,
            );
            if costs.get(&neighbor).is_none_or(|current| next < *current) {
                costs.insert(neighbor, next);
                previous.insert(neighbor, cell);
                frontier.push(Reverse((next.0, next.1, neighbor)));
            }
        }
    };
    let mut path = vec![target];
    while let Some(&parent) = previous.get(path.last().expect("connector path is nonempty")) {
        path.push(parent);
    }
    path
}

fn reflection_orbit(board: BoardSize, cell: CellId) -> BTreeSet<CellId> {
    BTreeSet::from([
        cell,
        CellId::new(board.width() - 1 - cell.column, cell.row),
        CellId::new(cell.column, board.height() - 1 - cell.row),
        CellId::new(
            board.width() - 1 - cell.column,
            board.height() - 1 - cell.row,
        ),
    ])
}

#[cfg(test)]
#[path = "generated_v5_tests.rs"]
mod tests;
