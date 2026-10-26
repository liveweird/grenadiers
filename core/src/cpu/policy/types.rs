use crate::cpu::{Intent, MemoryKey};
use crate::game::{CellId, Position};

pub const MAX_HOLD_TICKS: u8 = 12;
pub const MAX_ROUTE_CELLS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PolicyError {
    IdentityMismatch,
    InvalidObservation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UtilityMemory {
    key: MemoryKey,
    route: [Option<CellId>; MAX_ROUTE_CELLS],
    route_len: u8,
    route_index: u8,
    planted_origin: Option<CellId>,
    refuge: Option<CellId>,
}

impl UtilityMemory {
    #[must_use]
    pub const fn new(key: MemoryKey) -> Self {
        Self {
            key,
            route: [None; MAX_ROUTE_CELLS],
            route_len: 0,
            route_index: 0,
            planted_origin: None,
            refuge: None,
        }
    }

    #[must_use]
    pub const fn key(self) -> MemoryKey {
        self.key
    }

    #[must_use]
    pub const fn route(&self) -> &[Option<CellId>; MAX_ROUTE_CELLS] {
        &self.route
    }

    #[must_use]
    pub const fn planted_origin(self) -> Option<CellId> {
        self.planted_origin
    }

    #[must_use]
    pub const fn refuge(self) -> Option<CellId> {
        self.refuge
    }

    pub(crate) fn from_route(key: MemoryKey, cells: &[CellId], planted: bool) -> Self {
        let mut memory = Self::new(key);
        let len = cells.len().min(MAX_ROUTE_CELLS);
        let mut index = 0;
        while index < len {
            memory.route[index] = Some(cells[index]);
            index += 1;
        }
        memory.route_len = len as u8;
        memory.refuge = cells.get(len.saturating_sub(1)).copied();
        if planted {
            memory.planted_origin = cells.first().copied();
            memory.route_index = 1;
        }
        memory
    }

    pub(crate) fn next(mut self, current: CellId, position: Position) -> Option<(CellId, Self)> {
        while self.route_index < self.route_len {
            let index = usize::from(self.route_index);
            let cell = self.route[index]?;
            if cell != current || !near_center(position, cell) {
                return Some((cell, self));
            }
            self.route_index += 1;
        }
        None
    }
}

fn near_center(position: Position, cell: CellId) -> bool {
    let center = Position::center(cell);
    position.x.abs_diff(center.x) <= 64 && position.y.abs_diff(center.y) <= 64
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlPlan {
    pub intent: Intent,
    pub hold_ticks: u8,
    pub next_memory: UtilityMemory,
}

impl ControlPlan {
    #[must_use]
    pub const fn stop(memory: UtilityMemory) -> Self {
        Self {
            intent: Intent::STOP,
            hold_ticks: 1,
            next_memory: memory,
        }
    }
}
