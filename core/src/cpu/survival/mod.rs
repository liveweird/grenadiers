//! Receding-horizon survival: one executable pulse and a bounded stopped lease.
//!
//! A lease certifies only its stated interval. It never grants authority to
//! execute a guidance suffix or to plant a bomb.

mod clear_gap;
mod compatibility;
mod deadline;
mod execution;
mod origin;
mod origin_planner;
mod planner;
mod plant_value;
mod route_execution;
mod steering;

pub use compatibility::can_admit_ordinary;
pub use execution::{CommittedPulse, PulseStatus};
pub use origin::PlantEscapeOrigin;
pub use planner::{ExecutionCertificate, SurvivalPlan, plan, plan_with_origin};
pub use route_execution::{CommittedRoute, RouteCommand, RouteStatus};

pub const MAX_PLAN_AGE_TICKS: u64 = 6;
pub const WORK_ALLOWANCE: u32 = 65_536;

#[cfg(test)]
mod deadline_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod route_tests;

#[cfg(test)]
mod clear_gap_tests;

#[cfg(test)]
mod origin_tests;
