use std::collections::{BTreeSet, VecDeque};

use super::types::CellId;

pub const ARENA_WIDTH: u16 = 15;
pub const ARENA_HEIGHT: u16 = 13;
pub const ARENA_CELL_COUNT: usize = ARENA_WIDTH as usize * ARENA_HEIGHT as usize;

const FIRST_CORNER: CellId = CellId::new(1, 1);
const SECOND_CORNER: CellId = CellId::new(13, 11);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Floor,
    Wall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArenaError {
    BorderMustBeWall(CellId),
    SpawnsMustBeOppositeCorners,
    DuplicateSpawn(CellId),
    SpawnMustBeFloor(CellId),
    ReservedCellMustBeFloor(CellId),
    ReservedCellHasBlock(CellId),
    BlockOutOfBounds(CellId),
    BlockMustBeOnFloor(CellId),
    FloorIsDisconnected(CellId),
}

/// The fixed-size terrain, soft-block layer, and spawn metadata for one round.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arena {
    terrain: [Terrain; ARENA_CELL_COUNT],
    blocks: BTreeSet<CellId>,
    spawns: [CellId; 2],
}

impl Arena {
    pub fn new(
        terrain: [Terrain; ARENA_CELL_COUNT],
        blocks: BTreeSet<CellId>,
        spawns: [CellId; 2],
    ) -> Result<Self, ArenaError> {
        let arena = Self {
            terrain,
            blocks,
            spawns,
        };
        arena.validate()?;
        Ok(arena)
    }

    #[must_use]
    pub const fn contains(&self, cell: CellId) -> bool {
        cell.column < ARENA_WIDTH && cell.row < ARENA_HEIGHT
    }

    #[must_use]
    pub fn terrain(&self, cell: CellId) -> Option<Terrain> {
        self.index(cell).map(|index| self.terrain[index])
    }

    #[must_use]
    pub const fn blocks(&self) -> &BTreeSet<CellId> {
        &self.blocks
    }

    #[must_use]
    pub const fn spawns(&self) -> [CellId; 2] {
        self.spawns
    }

    pub(crate) fn remove_blocks(&mut self, removed: &BTreeSet<CellId>) {
        self.blocks.retain(|cell| !removed.contains(cell));
    }

    fn validate(&self) -> Result<(), ArenaError> {
        for column in 0..ARENA_WIDTH {
            self.require_border_wall(CellId::new(column, 0))?;
            self.require_border_wall(CellId::new(column, ARENA_HEIGHT - 1))?;
        }
        for row in 1..ARENA_HEIGHT - 1 {
            self.require_border_wall(CellId::new(0, row))?;
            self.require_border_wall(CellId::new(ARENA_WIDTH - 1, row))?;
        }

        if self.spawns[0] == self.spawns[1] {
            return Err(ArenaError::DuplicateSpawn(self.spawns[0]));
        }
        if !self.spawns.contains(&FIRST_CORNER) || !self.spawns.contains(&SECOND_CORNER) {
            return Err(ArenaError::SpawnsMustBeOppositeCorners);
        }
        for spawn in self.spawns {
            if self.terrain(spawn) != Some(Terrain::Floor) {
                return Err(ArenaError::SpawnMustBeFloor(spawn));
            }
        }

        for cell in reserved_spawn_cells() {
            if self.terrain(cell) != Some(Terrain::Floor) {
                return Err(ArenaError::ReservedCellMustBeFloor(cell));
            }
            if self.blocks.contains(&cell) {
                return Err(ArenaError::ReservedCellHasBlock(cell));
            }
        }

        for &cell in &self.blocks {
            if !self.contains(cell) {
                return Err(ArenaError::BlockOutOfBounds(cell));
            }
            if self.terrain(cell) != Some(Terrain::Floor) {
                return Err(ArenaError::BlockMustBeOnFloor(cell));
            }
        }

        self.validate_floor_connectivity()
    }

    fn require_border_wall(&self, cell: CellId) -> Result<(), ArenaError> {
        if self.terrain(cell) == Some(Terrain::Wall) {
            Ok(())
        } else {
            Err(ArenaError::BorderMustBeWall(cell))
        }
    }

    fn validate_floor_connectivity(&self) -> Result<(), ArenaError> {
        let mut visited = BTreeSet::new();
        let mut pending = VecDeque::from([self.spawns[0]]);

        while let Some(cell) = pending.pop_front() {
            if !visited.insert(cell) {
                continue;
            }
            for neighbor in neighbors(cell) {
                if self.terrain(neighbor) == Some(Terrain::Floor) && !visited.contains(&neighbor) {
                    pending.push_back(neighbor);
                }
            }
        }

        for row in 0..ARENA_HEIGHT {
            for column in 0..ARENA_WIDTH {
                let cell = CellId::new(column, row);
                if self.terrain(cell) == Some(Terrain::Floor) && !visited.contains(&cell) {
                    return Err(ArenaError::FloorIsDisconnected(cell));
                }
            }
        }
        Ok(())
    }

    fn index(&self, cell: CellId) -> Option<usize> {
        self.contains(cell)
            .then_some(usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column))
    }
}

fn neighbors(cell: CellId) -> impl Iterator<Item = CellId> {
    let mut neighbors = [None; 4];
    neighbors[0] = cell
        .column
        .checked_sub(1)
        .map(|column| CellId::new(column, cell.row));
    neighbors[1] = cell
        .column
        .checked_add(1)
        .map(|column| CellId::new(column, cell.row));
    neighbors[2] = cell
        .row
        .checked_sub(1)
        .map(|row| CellId::new(cell.column, row));
    neighbors[3] = cell
        .row
        .checked_add(1)
        .map(|row| CellId::new(cell.column, row));
    neighbors.into_iter().flatten()
}

fn reserved_spawn_cells() -> [CellId; 6] {
    [
        FIRST_CORNER,
        CellId::new(2, 1),
        CellId::new(1, 2),
        SECOND_CORNER,
        CellId::new(12, 11),
        CellId::new(13, 10),
    ]
}

#[cfg(test)]
#[path = "arena_tests.rs"]
mod tests;
