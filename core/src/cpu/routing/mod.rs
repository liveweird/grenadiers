//! Bounded time-aware routing over recipient-visible hazard forecasts.

mod motion;
mod pulse;
mod pulse_geometry;
mod route;
mod route_checks;
mod route_expand;
mod route_replay;
mod route_time;
mod route_types;
mod safety;
mod types;
mod validation;

pub use route_types::{
    MAX_ROUTE_EXPANSIONS, MAX_ROUTE_STEPS, RouteCertificate, RouteLane, RouteLimits, RouteRequest,
    RouteStep,
};
pub use types::{PoseBounds, PulseLease, PulseRequest, RouteAssurance, RouteFailure, RouteMode};

pub use pulse::certify_pulse;
pub use route::plan_route;

#[cfg(test)]
#[path = "tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "boundary_tests.rs"]
mod boundary_tests;

#[cfg(test)]
#[path = "route_tests.rs"]
mod route_tests;
