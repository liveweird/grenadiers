use crate::game::{BlastKind, CellId, EntityId, Tick};

use super::observation::Unknown;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObservedBombTrigger {
    Timed { remaining_ticks: u16, paused: bool },
    Remote,
    Contact,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ObservedBomb {
    pub id: EntityId,
    pub placed_at: Tick,
    pub cell: CellId,
    pub owner: EntityId,
    pub range: u16,
    pub trigger: ObservedBombTrigger,
    pub blast: BlastKind,
    pub unknown: UnknownBombFacts,
}

/// Authoritative bomb bookkeeping intentionally absent from recipient views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownBombFacts {
    pub motion: Unknown,
    pub interaction_history: Unknown,
}

impl UnknownBombFacts {
    pub const ALL: Self = Self {
        motion: Unknown::Unknown,
        interaction_history: Unknown::Unknown,
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BombKnowledge {
    /// The recipient is not allowed to know whether any bombs exist.
    Unavailable,
    /// Complete recipient-visible bomb information, which may be empty.
    Visible(Vec<ObservedBomb>),
}
