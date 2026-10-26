//! Pure, transport-neutral contracts for server-controlled players.
//!
//! Policies consume recipient-filtered [`Observation`] values rather than
//! authoritative simulation state and return only ordinary [`Intent`] values.

pub mod forecast;
mod identity;
mod intent;
mod memory;
mod observation;
pub mod policy;
mod random;
pub mod routing;
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
