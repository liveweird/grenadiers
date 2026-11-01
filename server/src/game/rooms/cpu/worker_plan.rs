use super::PlanJob;
use grenadiers_core::cpu::{
    MobContactLease, WorkBudget,
    policy::PolicyError,
    survival::{
        SurvivalPlan, plan_counter_pressure_v29_with_origin, plan_farm_origins_v38_with_origin,
        plan_farm_refuges_v36_with_origin, plan_hard_farm_origins_v39_with_origin,
        plan_hard_farm_refuges_v37_with_origin, plan_hard_pickup_progress_v32_with_origin,
        plan_hard_v28_with_origin, plan_lairs_with_origin, plan_pickup_progress_v31_with_origin,
        plan_refuge_guidance_v33_with_origin, plan_refuge_guidance_v34_with_origin,
        plan_reliable_v27_with_origin, plan_respawn_v22_with_origin, plan_respawn_v23_with_origin,
        plan_respawn_v24_with_origin, plan_respawn_v25_with_origin, plan_respawn_with_origin,
        plan_spike_pits_v26_with_origin, plan_with_origin,
    },
};
use grenadiers_core::room::CpuPolicy;

pub(super) fn plan(
    job: &mut PlanJob,
    work: &mut WorkBudget,
) -> Result<(SurvivalPlan, Option<MobContactLease>), PolicyError> {
    if job.observation.parts().rules.version == 10
        || matches!(
            job.policy,
            CpuPolicy::UtilityV26
                | CpuPolicy::UtilityV27
                | CpuPolicy::UtilityV28
                | CpuPolicy::UtilityV29
                | CpuPolicy::UtilityV31
                | CpuPolicy::UtilityV32
                | CpuPolicy::UtilityV33
                | CpuPolicy::UtilityV34
                | CpuPolicy::UtilityV35
                | CpuPolicy::UtilityV36
                | CpuPolicy::UtilityV37
                | CpuPolicy::UtilityV38
                | CpuPolicy::UtilityV39
        )
    {
        let plan = if job.policy == CpuPolicy::UtilityV38 {
            plan_farm_origins_v38_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV39 {
            plan_hard_farm_origins_v39_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV36 {
            plan_farm_refuges_v36_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV37 {
            plan_hard_farm_refuges_v37_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV33 {
            plan_refuge_guidance_v33_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        // Policy35 adds controller handoff; reuse the exact policy34 planner.
        } else if matches!(job.policy, CpuPolicy::UtilityV34 | CpuPolicy::UtilityV35) {
            plan_refuge_guidance_v34_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV31 {
            plan_pickup_progress_v31_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV32 {
            plan_hard_pickup_progress_v32_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV29 {
            plan_counter_pressure_v29_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV28 {
            plan_hard_v28_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV27 {
            plan_reliable_v27_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV26 {
            plan_spike_pits_v26_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else {
            plan_respawn_v25_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        };
        plan.map(|plan| (plan.survival, plan.contact))
    } else if job.observation.parts().rules.version == 9 {
        let plan = if job.policy == CpuPolicy::UtilityV25 {
            plan_respawn_v25_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV24 {
            plan_respawn_v24_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV23 {
            plan_respawn_v23_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else if job.policy == CpuPolicy::UtilityV22 {
            plan_respawn_v22_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        } else {
            plan_respawn_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut job.rng,
                work,
            )
        };
        plan.map(|plan| (plan.survival, plan.contact))
    } else if job.observation.parts().rules.version == 8 {
        plan_lairs_with_origin(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut job.rng,
            work,
        )
        .map(|plan| (plan.survival, plan.contact))
    } else {
        plan_with_origin(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut job.rng,
            work,
        )
        .map(|plan| (plan, None))
    }
}
