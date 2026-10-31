//! Receding-horizon survival: one executable pulse and a bounded stopped lease.
//!
//! A lease certifies only its stated interval. It never grants authority to
//! execute a guidance suffix or to plant a bomb.

mod bracelet;
mod clear_gap;
mod compatibility;
mod deadline;
mod execution;
mod lair_planner;
mod lair_v23;
mod manual_execution;
mod origin;
mod origin_planner;
mod pickup_handoff;
mod planner;
mod plant_tactic;
mod plant_value;
mod respawn_disabled;
mod respawn_v22;
mod route_execution;
mod steering;

pub use compatibility::can_admit_ordinary;
pub use execution::{CommittedPulse, PulseStatus};
pub use lair_planner::{LairSurvivalPlan, plan_lairs, plan_lairs_with_origin};
pub use manual_execution::ExecutionCertificate;
pub use origin::PlantEscapeOrigin;
pub use planner::{SurvivalPlan, plan, plan_with_origin};
pub(crate) use plant_value::ObservedBlastGeometry;
pub use plant_value::strategic_plant_still_targets;
pub use respawn_v22::{
    plan_respawn, plan_respawn_v22_with_origin, plan_respawn_v23_with_origin,
    plan_respawn_v24_with_origin, plan_respawn_with_origin,
};
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
mod bracelet_guard_tests;

#[cfg(test)]
mod bracelet_tests;

#[cfg(test)]
mod origin_tests;

#[cfg(test)]
mod multiplayer_rival_tests;

#[cfg(test)]
mod storm_planner_tests;

#[cfg(test)]
mod lair_planner_tests;

#[cfg(test)]
mod lair_origin_tests;

#[cfg(test)]
mod lair_pickup_execution_tests;

#[cfg(test)]
mod rules_v9_occupancy_tests;

#[cfg(test)]
mod rules_v9_strategy_tests;
