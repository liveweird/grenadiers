//! Bounded time-aware routing over recipient-visible hazard forecasts.

mod motion;
mod plant;
mod plant_types;
mod pulse;
mod pulse_geometry;
mod route;
mod route_checks;
mod route_expand;
mod route_priority;
mod route_replay;
mod route_time;
mod route_types;
mod safety;
mod types;
mod validation;

pub use plant_types::{PlannedBomb, PlantCertificate, PlantFailure, PlantPresence, PlantRequest};
pub use route_types::{
    MAX_ROUTE_EXPANSIONS, MAX_ROUTE_STEPS, RouteCertificate, RouteLane, RouteLimits, RouteRequest,
    RouteStep,
};
pub use types::{PoseBounds, PulseLease, PulseRequest, RouteAssurance, RouteFailure, RouteMode};

pub use plant::{plan_ordinary_plant, plan_timed_weapon_plant};
pub use pulse::certify_pulse;
pub(crate) use pulse::{PulseOriginGrace, certify_escape_pulse};
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
#[path = "motion_regression_tests.rs"]
mod motion_regression_tests;

#[cfg(test)]
#[path = "plant_tests.rs"]
pub(crate) mod plant_tests;

#[cfg(test)]
#[path = "plant_failure_tests.rs"]
mod plant_failure_tests;

#[cfg(test)]
#[path = "weapon_plant_tests.rs"]
mod weapon_plant_tests;

#[cfg(test)]
#[path = "wall_pass_plant_tests.rs"]
mod wall_pass_plant_tests;
