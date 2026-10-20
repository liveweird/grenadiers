use super::types::{CellId, EntityId, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombTrigger {
    Timed { detonates_at: Tick },
    Remote,
    Contact,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BlastKind {
    #[default]
    Standard,
    Piercing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombError {
    DeadlineBeforePlacement,
}

/// An owned bomb with placement-time properties captured for later detonation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bomb {
    pub(crate) id: EntityId,
    pub(crate) cell: CellId,
    pub(crate) owner: EntityId,
    pub(crate) range: u16,
    pub(crate) placed_at: Tick,
    pub(crate) trigger: BombTrigger,
    blast: BlastKind,
}

impl Bomb {
    pub(crate) const fn new(
        id: EntityId,
        cell: CellId,
        owner: EntityId,
        range: u16,
        placed_at: Tick,
        trigger: BombTrigger,
    ) -> Result<Self, BombError> {
        Self::new_with_blast(
            id,
            cell,
            owner,
            range,
            placed_at,
            trigger,
            BlastKind::Standard,
        )
    }

    pub(crate) const fn new_with_blast(
        id: EntityId,
        cell: CellId,
        owner: EntityId,
        range: u16,
        placed_at: Tick,
        trigger: BombTrigger,
        blast: BlastKind,
    ) -> Result<Self, BombError> {
        if let BombTrigger::Timed { detonates_at } = trigger
            && detonates_at.0 < placed_at.0
        {
            return Err(BombError::DeadlineBeforePlacement);
        }
        Ok(Self {
            id,
            cell,
            owner,
            range,
            placed_at,
            trigger,
            blast,
        })
    }

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
    pub const fn trigger(self) -> BombTrigger {
        self.trigger
    }

    #[must_use]
    pub const fn blast(self) -> BlastKind {
        self.blast
    }

    #[must_use]
    pub const fn detonates_at(self) -> Option<Tick> {
        match self.trigger {
            BombTrigger::Timed { detonates_at } => Some(detonates_at),
            BombTrigger::Remote | BombTrigger::Contact => None,
        }
    }
}

#[cfg(test)]
#[path = "bombs_tests.rs"]
mod tests;
