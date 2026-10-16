use super::types::{CellId, EntityId, Tick};

/// An owned bomb with placement-time properties captured for later detonation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bomb {
    pub(crate) id: EntityId,
    pub(crate) cell: CellId,
    pub(crate) owner: EntityId,
    pub(crate) range: u16,
    pub(crate) placed_at: Tick,
    pub(crate) detonates_at: Tick,
}

impl Bomb {
    #[must_use]
    pub const fn id(self) -> EntityId {
        self.id
    }

    #[must_use]
    pub const fn cell(self) -> CellId {
        self.cell
    }

    #[must_use]
    pub const fn owner(self) -> EntityId {
        self.owner
    }

    #[must_use]
    pub const fn range(self) -> u16 {
        self.range
    }

    #[must_use]
    pub const fn placed_at(self) -> Tick {
        self.placed_at
    }

    #[must_use]
    pub const fn detonates_at(self) -> Tick {
        self.detonates_at
    }
}
