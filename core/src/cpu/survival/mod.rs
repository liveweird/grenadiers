//! Receding-horizon survival: one executable pulse and a bounded stopped lease.
//!
//! A lease certifies only its stated interval. It never grants authority to
//! execute a guidance suffix or to plant a bomb.

mod bracelet;
mod clear_gap;
mod compatibility;
mod contact_plant;
mod continuation_compatibility;
mod continuation_execution;
mod continuation_pending;
mod continuation_planner;
mod continuation_selection;
mod deadline;
mod execution;
mod joint_refuge;
mod lair_planner;
mod lair_v23;
mod manual_execution;
mod ordinary_planner;
mod origin;
mod origin_planner;
mod pickup_handoff;
mod pickup_progress;
mod planner;
mod plant_tactic;
mod plant_value;
mod punch_tactic;
mod refuge_guidance;
mod refuge_handoff;
mod respawn_disabled;
mod respawn_v22;
mod route_execution;
mod route_transitions;
mod steering;
mod strategic_recovery;
mod teleporter_execution;
mod teleporter_execution_compat;
mod teleporter_execution_types;
mod teleporter_recovery;
mod teleporter_recovery_types;

pub use compatibility::can_admit_ordinary;
pub use contact_plant::PlantContactCertificate;
#[cfg(test)]
pub(crate) use contact_plant::contact_scene_candidate_for_test;
pub use continuation_execution::{
    CommittedContinuation, ContinuationCommand, ContinuationPublication, ContinuationStatus,
    PreparedContinuation, PreparedRenewal,
};
pub use continuation_pending::PendingContinuation;
#[cfg(test)]
pub(crate) use continuation_planner::certify_continuation;
pub use continuation_selection::{
    OriginRecoveryChoice, continuation_recovery_memory, continuation_recovery_stop,
    plan_lairs as plan_continuation_lairs,
};
pub use execution::{CommittedPulse, PulseStatus};
pub use lair_planner::{LairSurvivalPlan, plan_lairs, plan_lairs_with_origin};
pub use manual_execution::ExecutionCertificate;
pub use origin::PlantEscapeOrigin;
pub use pickup_progress::{
    plan_hard_pickup_progress_v32_with_origin, plan_pickup_progress_v31_with_origin,
};
pub use planner::{plan, plan_with_origin};
pub(crate) use plant_value::ObservedBlastGeometry;
pub use plant_value::strategic_plant_still_targets;
pub use refuge_guidance::{
    plan_blocked_births_v42_with_origin, plan_contact_topology_v40_with_origin,
    plan_farm_origins_v38_with_origin, plan_farm_refuges_v36_with_origin,
    plan_hard_blocked_births_v43_with_origin, plan_hard_contact_topology_v41_with_origin,
    plan_hard_farm_origins_v39_with_origin, plan_hard_farm_refuges_v37_with_origin,
    plan_refuge_guidance_v33_with_origin, plan_refuge_guidance_v34_with_origin,
};
pub use refuge_handoff::TerminalRefugeHandoff;
#[cfg(test)]
pub(crate) use reliable_v27_oil_tests::observe_with_lairs;
pub use respawn_v22::{
    plan_counter_pressure_v29_with_origin, plan_hard_v28_with_origin,
    plan_reliable_v27_with_origin, plan_respawn, plan_respawn_v22_with_origin,
    plan_respawn_v23_with_origin, plan_respawn_v24_with_origin, plan_respawn_v25_with_origin,
    plan_respawn_with_origin, plan_spike_pits_v26_with_origin,
};
pub use route_execution::{CommittedRoute, RouteCommand, RouteStatus};
pub use teleporter_execution::TELEPORTER_ADMISSION_ALLOWANCE;
pub use teleporter_execution_types::{
    CommittedTeleporterStep, PreparedTeleporterStep, TeleporterOutcome, TeleporterStepCommand,
    TeleporterStepStatus,
};
pub use teleporter_recovery::{TeleporterRecoveryChoice, select_teleporter_recovery};
pub use teleporter_recovery_types::TeleporterStepPlan;

pub const MAX_PLAN_AGE_TICKS: u64 = 6;
pub const WORK_ALLOWANCE: u32 = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SurvivalPlan {
    pub control: crate::cpu::policy::ControlPlan,
    pub lease: Option<crate::cpu::routing::PulseLease>,
    pub execution: Option<ExecutionCertificate>,
}

#[cfg(test)]
mod deadline_tests;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod route_tests;

#[cfg(test)]
mod reliable_v27_oil_tests;

#[cfg(test)]
mod observation_test_adapter;

#[cfg(test)]
mod hard_v28_tactic_tests;

#[cfg(test)]
mod counter_pressure_v29_tests;

#[cfg(test)]
mod counter_pressure_v29_acceptance_tests;

#[cfg(test)]
mod continuation_tests;

#[cfg(test)]
mod continuation_authority_tests;
#[cfg(test)]
mod continuation_execution_guard_tests;
#[cfg(test)]
mod continuation_execution_tests;

#[cfg(test)]
mod continuation_test_fixture;

#[cfg(test)]
mod continuation_scene_fixture;

#[cfg(test)]
mod continuation_guard_tests;

#[cfg(test)]
mod continuation_selection_tests;

#[cfg(test)]
mod pickup_stall_tests;

#[cfg(test)]
mod pickup_progress_execution_tests;

#[cfg(test)]
mod refuge_centering_test_fixture;

#[cfg(test)]
mod refuge_centering_tests;

#[cfg(test)]
mod refuge_centering_episode_tests;

#[cfg(test)]
mod farm_progress_tests;

#[cfg(test)]
mod farm_origin_tests;

#[cfg(test)]
mod joint_refuge_execution_tests;

#[cfg(test)]
mod refuge_handoff_tests;

#[cfg(test)]
mod refuge_handoff_guard_tests;

#[cfg(test)]
mod reliable_v27_oil_guard_tests;

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

#[cfg(test)]
mod spike_pit_execution_tests;

#[cfg(test)]
mod spike_pit_plant_execution_tests;

#[cfg(test)]
mod teleporter_execution_tests;

#[cfg(test)]
mod teleporter_execution_rejection_tests;

#[cfg(test)]
mod teleporter_execution_timing_tests;

#[cfg(test)]
mod teleporter_execution_environment_tests;
