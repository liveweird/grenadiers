//! Bounded time-aware routing over recipient-visible hazard forecasts.

mod bracelet_guard;
// Explicitly inactive proof checkpoint. Remove these narrow allowances when
// the bounded planner/executor callers land, before gameplay activation.
#[cfg_attr(not(test), allow(dead_code))]
mod continuation;
mod continuation_candidates;
#[cfg_attr(not(test), allow(dead_code))]
mod continuation_proof;
mod continuation_renewal;
#[cfg_attr(not(test), allow(dead_code))]
mod continuation_types;
mod motion;
mod pickup_pulse;
mod plant;
mod plant_types;
mod pulse;
mod pulse_geometry;
mod pulse_support;
mod punch;
mod punch_types;
mod respawn_pulse;
mod route;
mod route_checks;
mod route_expand;
mod route_hints;
mod route_priority;
mod route_replay;
mod route_time;
mod route_types;
mod safety;
mod spike_pits;
pub(crate) mod storm;
mod types;
mod validation;

pub use plant_types::{PlannedBomb, PlantCertificate, PlantFailure, PlantPresence, PlantRequest};
pub use punch_types::{PunchCertificate, PunchFailure, PunchOutcome, PunchRequest};
pub use route_types::{
    MAX_ROUTE_EXPANSIONS, MAX_ROUTE_STEPS, RouteCertificate, RouteLane, RouteLimits, RouteRequest,
    RouteStep,
};
pub use spike_pits::SpikePitGuard;
pub use types::{PoseBounds, PulseLease, PulseRequest, RouteAssurance, RouteFailure, RouteMode};

pub(crate) use bracelet_guard::BraceletGuard;
#[allow(unused_imports)] // Inactive survival adapter is exercised by focused tests.
pub(crate) use continuation::certify_continuation;
#[cfg(test)]
pub(crate) use continuation::certify_nominal_for_test;
pub(crate) use continuation_candidates::continuation_candidates;
pub(crate) use continuation_renewal::{
    CONTINUATION_RENEWAL_ALLOWANCE, PreparedFrame, PreparedGrant,
};
pub(crate) use continuation_types::{ContinuationCertificate, ContinuationRequest, NominalLeg};

pub(crate) use pickup_pulse::{certify_lair_contact_pulse, certify_pickup_pulse};
pub(crate) use plant::{
    derive_bomb_modifiers, plan_lair_ordinary_plant, plan_lair_ordinary_plant_to_refuge,
    plan_lair_timed_weapon_plant,
};
pub use plant::{plan_ordinary_plant, plan_timed_weapon_plant};
pub(crate) use pulse::{PulseOriginGrace, certify_escape_pulse, certify_lair_escape_pulse};
pub use pulse::{certify_lair_pulse, certify_pulse};
pub use punch::plan_punch;
pub(crate) use respawn_pulse::certify_respawn_pulse;
pub(crate) use route::plan_respawn_route;
pub use route::plan_route;

pub(crate) const CERTIFIED_MOVE_UPDATES: u8 = 12;
pub(crate) const MIN_FINAL_STOP_UPDATES: u64 = 10;

#[cfg(test)]
#[path = "tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "boundary_tests.rs"]
mod boundary_tests;

#[cfg(test)]
#[path = "route_tests.rs"]
mod route_tests;

#[cfg(test)]
mod route_terminal_tests;

#[cfg(test)]
#[path = "motion_regression_tests.rs"]
mod motion_regression_tests;

#[cfg(test)]
mod continuation_tests;

#[cfg(test)]
#[path = "plant_tests.rs"]
pub(crate) mod plant_tests;

#[cfg(test)]
#[path = "plant_failure_tests.rs"]
mod plant_failure_tests;

#[cfg(test)]
#[path = "punch_tests.rs"]
mod punch_tests;

#[cfg(test)]
mod multiplayer_punch_tests;

#[cfg(test)]
#[path = "weapon_plant_tests.rs"]
mod weapon_plant_tests;

#[cfg(test)]
#[path = "wall_pass_plant_tests.rs"]
mod wall_pass_plant_tests;

#[cfg(test)]
mod storm_tests;

#[cfg(test)]
mod storm_execution_tests;
