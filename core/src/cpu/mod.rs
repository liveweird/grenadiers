//! Pure, transport-neutral contracts for server-controlled players.
//!
//! Policies consume recipient-filtered [`Observation`] values rather than
//! authoritative simulation state and return only ordinary [`Intent`] values.

mod capabilities;
pub mod forecast;
mod identity;
mod intent;
mod memory;
mod observation;
pub mod policy;
mod random;
pub mod routing;
pub mod survival;
mod work;

pub(crate) fn rules_config(observation: &Observation) -> Option<crate::game::RulesConfig> {
    match observation.parts().rules.version {
        5 => Some(crate::game::RulesConfig::expanded_v5(
            observation.parts().rules.settings,
        )),
        6 => Some(crate::game::RulesConfig::multiplayer_v6(
            observation.parts().rules.settings,
        )),
        _ => None,
    }
}

pub(crate) const fn observed_bomb_capacity(rules_version: u16) -> usize {
    if rules_version == 6 {
        MAX_MULTIPLAYER_OBSERVED_BOMBS
    } else {
        MAX_OBSERVED_BOMBS
    }
}

pub use identity::{
    AdmissionContext, DecisionAdmission, DecisionId, RejectionReason, WorldId, admit_decision,
};
pub use intent::{Intent, ManualIntent, MovementIntent};
pub use memory::{
    CpuMemory, MAX_MEMORY_HISTORY, MemoryEntry, MemoryError, MemoryKey, MemoryUpdate,
};
pub use observation::{
    BombKnowledge, MAX_MULTIPLAYER_OBSERVED_BOMBS, MAX_OBSERVED_BOMBS, MAX_OBSERVED_EFFECTS,
    Observation, ObservationError, ObservationParts, ObservedActor, ObservedArena, ObservedBomb,
    ObservedBombTrigger, ObservedEffect, ObservedFlame, ObservedOil, ObservedPickup, ObservedRules,
    Unknown, UnknownActorFacts, UnknownBombFacts, VisibleActor,
};
pub use random::{CpuRng, RandomError};
pub use work::{WorkBudget, WorkError};

#[cfg(test)]
mod observation_multiplayer_tests;
#[cfg(test)]
mod tests;
