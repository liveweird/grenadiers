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
mod observation_bombs;
mod observation_environment;
mod observation_lairs;
mod observation_respawn_lairs;
mod observation_spike_pits;
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
        9 => Some(crate::game::RulesConfig::environment_v9(
            observation.parts().rules.settings,
            observation.environment()?.storm_cloud_config(),
            observation
                .respawn_lairs()
                .map_or(crate::room::RespawnLairConfig::Disabled, |lairs| {
                    lairs.config()
                }),
        )),
        10 => Some(crate::game::RulesConfig::environment_v10(
            observation.parts().rules.settings,
            observation.environment()?.storm_cloud_config(),
            observation
                .respawn_lairs()
                .map_or(crate::room::RespawnLairConfig::Disabled, |lairs| {
                    lairs.config()
                }),
            observation
                .spike_pits()
                .map_or(crate::room::SpikePitConfig::Disabled, |pits| pits.config()),
        )),
        _ => None,
    }
}

pub(crate) const fn observed_bomb_capacity(rules_version: u16) -> usize {
    if matches!(rules_version, 9 | 10) {
        crate::game::arena::ARENA_CELL_COUNT
    } else if rules_version >= 6 {
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
    ObservedOil, ObservedPickup, ObservedRespawnLairs, ObservedRules, ObservedSpikePits, Unknown,
    UnknownActorFacts, UnknownBombFacts, VisibleActor,
};
pub use random::{CpuRng, RandomError};
pub use work::{WorkBudget, WorkError};

#[cfg(test)]
mod observation_multiplayer_tests;
#[cfg(test)]
mod observation_spike_pits_tests;
#[cfg(test)]
mod rules_v8_boundary_tests;
#[cfg(test)]
mod rules_v9_episode_tests;

#[cfg(test)]
mod rules_v9_generated_tests;
#[cfg(test)]
mod tests;
