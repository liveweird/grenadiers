//! Pure, transport-neutral contracts for server-controlled players.
//!
//! Policies consume recipient-filtered [`Observation`] values rather than
//! authoritative simulation state and return only ordinary [`Intent`] values.

mod capabilities;
pub mod forecast;
mod identity;
mod intent;
mod lair_contact;
mod memory;
mod observation;
mod observation_environment;
mod observation_lairs;
mod observation_validation;
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
        7 => Some(crate::game::RulesConfig::environment_v7(
            observation.parts().rules.settings,
            observation.environment()?.storm_cloud_config(),
        )),
        8 => Some(crate::game::RulesConfig::environment_v8(
            observation.parts().rules.settings,
            observation.environment()?.storm_cloud_config(),
            observation.lairs()?.config(),
        )),
        _ => None,
    }
}

pub(crate) const fn observed_bomb_capacity(rules_version: u16) -> usize {
    if rules_version >= 6 {
        MAX_MULTIPLAYER_OBSERVED_BOMBS
    } else {
        MAX_OBSERVED_BOMBS
    }
}

pub use identity::{
    AdmissionContext, DecisionAdmission, DecisionId, RejectionReason, WorldId, admit_decision,
};
pub use intent::{Intent, ManualIntent, MovementIntent};
pub use lair_contact::{MobContactLease, has_urgent_lair_threat};
pub use memory::{
    CpuMemory, MAX_MEMORY_HISTORY, MemoryEntry, MemoryError, MemoryKey, MemoryUpdate,
};
pub use observation::{
    BombKnowledge, MAX_MULTIPLAYER_OBSERVED_BOMBS, MAX_OBSERVED_BOMBS, MAX_OBSERVED_EFFECTS,
    Observation, ObservationError, ObservationParts, ObservedActor, ObservedArena, ObservedBomb,
    ObservedBombTrigger, ObservedEffect, ObservedEnvironment, ObservedFlame, ObservedLairs,
    ObservedOil, ObservedPickup, ObservedRules, Unknown, UnknownActorFacts, UnknownBombFacts,
    VisibleActor,
};
pub use random::{CpuRng, RandomError};
pub use work::{WorkBudget, WorkError};

#[cfg(test)]
mod observation_multiplayer_tests;
#[cfg(test)]
mod rules_v8_boundary_tests;
#[cfg(test)]
mod tests;
