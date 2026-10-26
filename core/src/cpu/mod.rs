//! Pure, transport-neutral contracts for server-controlled players.
//!
//! This module deliberately contains no policy or authoritative simulation state.
//! A server adapter projects a recipient-filtered view into [`Observation`], and a
//! policy may return only an ordinary [`Intent`].

mod identity;
mod intent;
mod memory;
mod observation;
mod random;
mod work;

pub use identity::{
    AdmissionContext, DecisionAdmission, DecisionId, RejectionReason, WorldId, admit_decision,
};
pub use intent::{Intent, ManualIntent, MovementIntent};
pub use memory::{
    CpuMemory, MAX_MEMORY_HISTORY, MemoryEntry, MemoryError, MemoryKey, MemoryUpdate,
};
pub use observation::{
    BombKnowledge, MAX_OBSERVED_BOMBS, MAX_OBSERVED_EFFECTS, Observation, ObservationError,
    ObservationParts, ObservedActor, ObservedArena, ObservedBomb, ObservedBombTrigger,
    ObservedEffect, ObservedFlame, ObservedOil, ObservedPickup, ObservedRules, Unknown,
    UnknownActorFacts, UnknownBombFacts, VisibleActor,
};
pub use random::{CpuRng, RandomError};
pub use work::{WorkBudget, WorkError};

#[cfg(test)]
mod tests;
