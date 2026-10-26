//! Deterministic, bounded UtilityV1 policy.

mod execution;
mod hazards;
mod movement;
mod planner;
mod types;

pub use execution::follow_committed_escape;
pub use planner::plan;
pub use types::{ControlPlan, MAX_HOLD_TICKS, MAX_ROUTE_CELLS, PolicyError, UtilityMemory};

#[cfg(test)]
mod committed_execution_sim_tests;
#[cfg(test)]
mod execution_tests;
#[cfg(test)]
mod simulation_tests;
#[cfg(test)]
mod tests;
