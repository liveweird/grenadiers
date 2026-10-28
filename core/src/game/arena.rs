use std::collections::{BTreeSet, VecDeque};

use super::types::CellId;
use crate::room::BoardSize;

pub const ARENA_WIDTH: u16 = 15;
pub const ARENA_HEIGHT: u16 = 13;
pub const ARENA_CELL_COUNT: usize = ARENA_WIDTH as usize * ARENA_HEIGHT as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terrain {
    Floor,
    Wall,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArenaError {
    InvalidTerrainLength { expected: usize, actual: usize },
    BorderMustBeWall(CellId),
    SpawnsMustBeOppositeCorners,
    InvalidSpawnCount(usize),
    SpawnMustBeInteriorCorner(CellId),
    DuplicateSpawn(CellId),
    SpawnMustBeFloor(CellId),
    ReservedCellMustBeFloor(CellId),
    ReservedCellHasBlock(CellId),
    BlockOutOfBounds(CellId),
    BlockMustBeOnFloor(CellId),
    FloorIsDisconnected(CellId),
}

/// The checked terrain, soft-block layer, and spawn metadata for one round.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arena {
    board: BoardSize,
    terrain: Vec<Terrain>,
    blocks: BTreeSet<CellId>,
    spawns: Vec<CellId>,
}

impl Arena {
    pub fn new(
        terrain: [Terrain; ARENA_CELL_COUNT],
        blocks: BTreeSet<CellId>,
        spawns: impl Into<Vec<CellId>>,
    ) -> Result<Self, ArenaError> {
        Self::new_for_board(
            BoardSize::FifteenByThirteen,
            terrain.to_vec(),
            blocks,
            spawns,
        )
    }

    pub fn new_for_board(
        board: BoardSize,
        terrain: Vec<Terrain>,
        blocks: BTreeSet<CellId>,
        spawns: impl Into<Vec<CellId>>,
    ) -> Result<Self, ArenaError> {
        if terrain.len() != board.cell_count() {
            return Err(ArenaError::InvalidTerrainLength {
                expected: board.cell_count(),
                actual: terrain.len(),
            });
        }
        let arena = Self {
            board,
            terrain,
            blocks,
            spawns: spawns.into(),
        };
        arena.validate()?;
        Ok(arena)
    }

    #[must_use]
    pub const fn contains(&self, cell: CellId) -> bool {
        cell.column < self.width() && cell.row < self.height()
    }

    #[must_use]
    pub const fn board(&self) -> BoardSize {
        self.board
    }

    #[must_use]
    pub const fn width(&self) -> u16 {
        self.board.width()
    }

    #[must_use]
    pub const fn height(&self) -> u16 {
        self.board.height()
    }

    #[must_use]
    pub const fn cell_count(&self) -> usize {
        self.board.cell_count()
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
    pub fn spawns(&self) -> &[CellId] {
        &self.spawns
    }

    pub(crate) fn remove_blocks(&mut self, removed: &BTreeSet<CellId>) {
        self.blocks.retain(|cell| !removed.contains(cell));
    }

    fn validate(&self) -> Result<(), ArenaError> {
        for column in 0..self.width() {
            self.require_border_wall(CellId::new(column, 0))?;
            self.require_border_wall(CellId::new(column, self.height() - 1))?;
        }
        for row in 1..self.height() - 1 {
            self.require_border_wall(CellId::new(0, row))?;
            self.require_border_wall(CellId::new(self.width() - 1, row))?;
        }

        let count = self.spawns.len();
        if !(2..=4).contains(&count) {
            return Err(ArenaError::InvalidSpawnCount(count));
        }
        let mut unique = BTreeSet::new();
        for &spawn in &self.spawns {
            if !unique.insert(spawn) {
                return Err(ArenaError::DuplicateSpawn(spawn));
            }
        }
        let corners = self.interior_corners();
        if count == 2 && (!self.spawns.contains(&corners[0]) || !self.spawns.contains(&corners[3]))
        {
            return Err(ArenaError::SpawnsMustBeOppositeCorners);
        }
        if count > 2 {
            for &spawn in &self.spawns {
                if !corners.contains(&spawn) {
                    return Err(ArenaError::SpawnMustBeInteriorCorner(spawn));
                }
            }
        }
        for &spawn in &self.spawns {
            if self.terrain(spawn) != Some(Terrain::Floor) {
                return Err(ArenaError::SpawnMustBeFloor(spawn));
            }
        }

        let mut reserved_spawns = self.spawns.clone();
        reserved_spawns.sort();
        for cell in reserved_spawns.into_iter().flat_map(reserved_spawn_cells) {
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

        for row in 0..self.height() {
            for column in 0..self.width() {
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
            .then_some(usize::from(cell.row) * usize::from(self.width()) + usize::from(cell.column))
    }

    const fn interior_corners(&self) -> [CellId; 4] {
        let right = self.width() - 2;
        let bottom = self.height() - 2;
        [
            CellId::new(1, 1),
            CellId::new(right, 1),
            CellId::new(1, bottom),
            CellId::new(right, bottom),
        ]
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

fn reserved_spawn_cells(spawn: CellId) -> [CellId; 3] {
    let horizontal = if spawn.column == 1 {
        CellId::new(spawn.column + 1, spawn.row)
    } else {
        CellId::new(spawn.column - 1, spawn.row)
    };
    let vertical = if spawn.row == 1 {
        CellId::new(spawn.column, spawn.row + 1)
    } else {
        CellId::new(spawn.column, spawn.row - 1)
    };
    [spawn, horizontal, vertical]
}

#[cfg(test)]
#[path = "arena_tests.rs"]
mod tests;
