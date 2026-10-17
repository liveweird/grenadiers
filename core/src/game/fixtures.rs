use std::collections::BTreeSet;

use super::{
    arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH, Arena, Terrain},
    types::CellId,
};

const PILLARS: [CellId; 30] = [
    CellId::new(2, 2),
    CellId::new(4, 2),
    CellId::new(6, 2),
    CellId::new(8, 2),
    CellId::new(10, 2),
    CellId::new(12, 2),
    CellId::new(2, 4),
    CellId::new(4, 4),
    CellId::new(6, 4),
    CellId::new(8, 4),
    CellId::new(10, 4),
    CellId::new(12, 4),
    CellId::new(2, 6),
    CellId::new(4, 6),
    CellId::new(6, 6),
    CellId::new(8, 6),
    CellId::new(10, 6),
    CellId::new(12, 6),
    CellId::new(2, 8),
    CellId::new(4, 8),
    CellId::new(6, 8),
    CellId::new(8, 8),
    CellId::new(10, 8),
    CellId::new(12, 8),
    CellId::new(2, 10),
    CellId::new(4, 10),
    CellId::new(6, 10),
    CellId::new(8, 10),
    CellId::new(10, 10),
    CellId::new(12, 10),
];

const SOFT_BLOCKS: [CellId; 30] = [
    CellId::new(5, 1),
    CellId::new(6, 1),
    CellId::new(7, 1),
    CellId::new(8, 1),
    CellId::new(9, 1),
    CellId::new(3, 2),
    CellId::new(5, 2),
    CellId::new(7, 2),
    CellId::new(9, 2),
    CellId::new(11, 2),
    CellId::new(1, 3),
    CellId::new(2, 3),
    CellId::new(4, 3),
    CellId::new(6, 3),
    CellId::new(7, 3),
    CellId::new(7, 9),
    CellId::new(8, 9),
    CellId::new(10, 9),
    CellId::new(12, 9),
    CellId::new(13, 9),
    CellId::new(3, 10),
    CellId::new(5, 10),
    CellId::new(7, 10),
    CellId::new(9, 10),
    CellId::new(11, 10),
    CellId::new(5, 11),
    CellId::new(6, 11),
    CellId::new(7, 11),
    CellId::new(8, 11),
    CellId::new(9, 11),
];

/// A deterministic preview arena with the classic pillar lattice and symmetric
/// soft blocks. Each spawn has a clear three-cell outer-corridor route.
#[must_use]
pub fn fixed_arena() -> Arena {
    Arena::new(
        classic_terrain(),
        BTreeSet::from(SOFT_BLOCKS),
        classic_spawns(),
    )
    .expect("the fixed arena fixture must remain valid")
}

pub(crate) fn classic_terrain() -> [Terrain; ARENA_CELL_COUNT] {
    let mut terrain = [Terrain::Floor; ARENA_CELL_COUNT];
    for row in 0..ARENA_HEIGHT {
        for column in 0..ARENA_WIDTH {
            if column == 0 || column == ARENA_WIDTH - 1 || row == 0 || row == ARENA_HEIGHT - 1 {
                terrain[index(CellId::new(column, row))] = Terrain::Wall;
            }
        }
    }
    for pillar in PILLARS {
        terrain[index(pillar)] = Terrain::Wall;
    }
    terrain
}

pub(crate) const fn classic_spawns() -> [CellId; 2] {
    [CellId::new(1, 1), CellId::new(13, 11)]
}

fn index(cell: CellId) -> usize {
    usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column)
}

#[cfg(test)]
#[path = "fixtures_tests.rs"]
mod tests;
