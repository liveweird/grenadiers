//! Deterministic, bounded UtilityV1 policy.

mod equipment;
mod execution;
mod goals;
mod hazards;
mod movement;
mod placement;
mod planner;
mod types;

pub use execution::{EscapeExecution, follow_checked_escape, follow_committed_escape};
pub use planner::plan;
pub(crate) use planner::{plan_lairs, plan_respawn, plan_respawn_v23, plan_respawn_v24};
pub use types::{
    ControlPlan, FAILED_TARGET_DEFER_TICKS, GoalKind, GoalMemory, IntentDebugSummary,
    MAX_HOLD_TICKS, MAX_RECENT_CELLS, MAX_ROUTE_CELLS, PolicyError, STRATEGIC_COMMITMENT_TICKS,
    STRATEGIC_PROGRESS_TICKS, StrategicIntent, StrategicObjectiveKind, StrategicPhase,
    StrategicSwitchReason, StrategicTarget, UtilityMemory,
};

#[cfg(test)]
mod committed_execution_sim_tests;
#[cfg(test)]
mod equipment_goal_tests;
#[cfg(test)]
mod execution_tests;
#[cfg(test)]
mod goal_game_tests;
#[cfg(test)]
mod goal_tests;
#[cfg(test)]
mod ordinary_turn_tests;
#[cfg(test)]
mod rules_v9_goal_tests;
#[cfg(test)]
mod simulation_tests;
#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod wall_pass_tests;
#[cfg(test)]
mod weapon_goal_tests;
