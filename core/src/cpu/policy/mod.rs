//! Deterministic, bounded UtilityV1 policy.

mod execution;
mod goals;
mod hazards;
mod movement;
mod placement;
mod planner;
mod types;

pub use execution::{EscapeExecution, follow_checked_escape, follow_committed_escape};
pub use planner::plan;
pub use types::{ControlPlan, MAX_HOLD_TICKS, MAX_ROUTE_CELLS, PolicyError, UtilityMemory};

#[cfg(test)]
mod committed_execution_sim_tests;
#[cfg(test)]
mod execution_tests;
#[cfg(test)]
mod goal_game_tests;
#[cfg(test)]
mod goal_tests;
#[cfg(test)]
mod simulation_tests;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod weapon_goal_tests;
