//! Bounded time-aware routing over recipient-visible hazard forecasts.

mod pulse;
mod pulse_geometry;
mod safety;
mod types;
mod validation;

pub use types::{PoseBounds, PulseLease, PulseRequest, RouteAssurance, RouteFailure, RouteMode};

pub use pulse::certify_pulse;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "boundary_tests.rs"]
mod boundary_tests;
