use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use grenadiers_core::room::CpuPolicy;

use super::{CpuPlanner, tests::spike_job};

#[test]
fn queued_worker_dispatches_cpu27_only_through_its_exact_policy() {
    let planner = CpuPlanner::without_workers();
    let (job, result) = spike_job(CpuPolicy::UtilityV27);
    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (1, 0));
    assert!(report.work[0] <= WORK_ALLOWANCE);
    let result = result.recv().unwrap();
    let planned = result.ordinary();
    assert!(planned.lease.is_some() || planned.execution.is_some());
}

#[test]
fn queued_worker_dispatches_cpu28_with_identical_core_control_rng_and_work() {
    use grenadiers_core::cpu::{WorkBudget, survival::plan_hard_v28_with_origin};
    let planner = CpuPlanner::without_workers();
    let (job, result) = spike_job(CpuPolicy::UtilityV28);
    let mut rng = job.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let direct = plan_hard_v28_with_origin(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut rng,
        &mut work,
    )
    .unwrap();
    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (1, 0));
    assert_eq!(report.work, vec![work.spent()]);
    assert!(report.work[0] <= WORK_ALLOWANCE);
    let queued = result.recv().unwrap();
    assert_eq!(queued.ordinary().control, direct.survival.control);
    assert_eq!(queued.rng.draws(), rng.draws());
}

#[test]
fn queued_worker_dispatches_cpu29_with_identical_core_control_rng_and_work() {
    use grenadiers_core::cpu::{WorkBudget, survival::plan_counter_pressure_v29_with_origin};
    let planner = CpuPlanner::without_workers();
    let (job, result) = spike_job(CpuPolicy::UtilityV29);
    let mut rng = job.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let direct = plan_counter_pressure_v29_with_origin(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut rng,
        &mut work,
    )
    .unwrap();
    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (1, 0));
    assert_eq!(report.work, vec![work.spent()]);
    assert!(report.work[0] <= WORK_ALLOWANCE);
    let queued = result.recv().unwrap();
    assert_eq!(queued.ordinary().control, direct.survival.control);
    assert_eq!(queued.rng.draws(), rng.draws());
}

#[test]
fn queued_worker_dispatches_pickup_profiles_with_identical_core_control_rng_and_work() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{
            plan_hard_pickup_progress_v32_with_origin, plan_pickup_progress_v31_with_origin,
        },
    };
    for policy in [CpuPolicy::UtilityV31, CpuPolicy::UtilityV32] {
        let planner_fn = if policy == CpuPolicy::UtilityV31 {
            plan_pickup_progress_v31_with_origin
        } else {
            plan_hard_pickup_progress_v32_with_origin
        };
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        let mut rng = job.rng.clone();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let direct = planner_fn(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut rng,
            &mut work,
        )
        .unwrap();
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert_eq!(report.work, vec![work.spent()]);
        assert!(report.work[0] <= WORK_ALLOWANCE);
        let queued = result.recv().unwrap();
        assert_eq!(queued.ordinary().control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}

#[test]
fn queued_worker_dispatches_refuge_profiles_with_identical_core_control_rng_and_work() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{plan_refuge_guidance_v33_with_origin, plan_refuge_guidance_v34_with_origin},
    };
    for policy in [
        CpuPolicy::UtilityV33,
        CpuPolicy::UtilityV34,
        CpuPolicy::UtilityV35,
    ] {
        let planner_fn = if policy == CpuPolicy::UtilityV33 {
            plan_refuge_guidance_v33_with_origin
        } else {
            plan_refuge_guidance_v34_with_origin
        };
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        let mut rng = job.rng.clone();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let direct = planner_fn(
            &job.observation,
            &job.memory,
            &job.hazard_memory,
            job.settled_stop_tick,
            job.escape_origin.as_ref(),
            &mut rng,
            &mut work,
        )
        .unwrap();
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert_eq!(report.work, vec![work.spent()]);
        assert!(report.work[0] <= WORK_ALLOWANCE);
        let queued = result.recv().unwrap();
        assert_eq!(queued.ordinary().control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}

#[test]
fn queued_worker_dispatches_farm_refuge_profiles_with_direct_parity() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{plan_farm_refuges_v36_with_origin, plan_hard_farm_refuges_v37_with_origin},
    };
    for policy in [CpuPolicy::UtilityV36, CpuPolicy::UtilityV37] {
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        let mut rng = job.rng.clone();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let direct = if policy == CpuPolicy::UtilityV36 {
            plan_farm_refuges_v36_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else {
            plan_hard_farm_refuges_v37_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        }
        .unwrap();
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert_eq!(report.work, vec![work.spent()]);
        assert!(report.work[0] <= WORK_ALLOWANCE);
        let queued = result.recv().unwrap();
        assert_eq!(queued.ordinary().control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}

#[test]
fn queued_worker_dispatches_farm_origin_profiles_with_direct_parity() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{plan_farm_origins_v38_with_origin, plan_hard_farm_origins_v39_with_origin},
    };
    for policy in [CpuPolicy::UtilityV38, CpuPolicy::UtilityV39] {
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        let mut rng = job.rng.clone();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let direct = if policy == CpuPolicy::UtilityV38 {
            plan_farm_origins_v38_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else {
            plan_hard_farm_origins_v39_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        }
        .unwrap();
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert_eq!(report.work, vec![work.spent()]);
        assert!(report.work[0] <= WORK_ALLOWANCE);
        let queued = result.recv().unwrap();
        assert_eq!(queued.ordinary().control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}

#[test]
fn queued_worker_dispatches_contact_topology_profiles_with_direct_parity() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{
            plan_blocked_births_v42_with_origin, plan_contact_topology_v40_with_origin,
            plan_hard_blocked_births_v43_with_origin, plan_hard_contact_topology_v41_with_origin,
        },
    };
    for policy in [
        CpuPolicy::UtilityV40,
        CpuPolicy::UtilityV41,
        CpuPolicy::UtilityV42,
        CpuPolicy::UtilityV43,
        CpuPolicy::UtilityV44,
        CpuPolicy::UtilityV45,
    ] {
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        let mut rng = job.rng.clone();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let direct = if matches!(policy, CpuPolicy::UtilityV42 | CpuPolicy::UtilityV44) {
            plan_blocked_births_v42_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else if matches!(policy, CpuPolicy::UtilityV43 | CpuPolicy::UtilityV45) {
            plan_hard_blocked_births_v43_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else if policy == CpuPolicy::UtilityV40 {
            plan_contact_topology_v40_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else {
            plan_hard_contact_topology_v41_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        }
        .unwrap();
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert_eq!(report.work, vec![work.spent()]);
        assert!(report.work[0] <= WORK_ALLOWANCE);
        let queued = result.recv().unwrap();
        assert_eq!(queued.ordinary().control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}

#[test]
fn queued_worker_keeps_hard57_and_hard58_archived_and_dispatches_hard59_exactly() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{
            TeleporterRecoveryChoice, plan_hard_corner_refuges_v57_with_origin,
            plan_hard_exit_pressure_v58_with_origin, plan_hard_farm_yield_v59_with_origin,
            select_teleporter_recovery_v50,
        },
    };
    for policy in [
        CpuPolicy::UtilityV57,
        CpuPolicy::UtilityV58,
        CpuPolicy::UtilityV59,
    ] {
        let planner = CpuPlanner::without_workers();
        let (job, result) = spike_job(policy);
        let mut rng = job.rng.clone();
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let direct = if policy == CpuPolicy::UtilityV57 {
            plan_hard_corner_refuges_v57_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else if policy == CpuPolicy::UtilityV58 {
            plan_hard_exit_pressure_v58_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        } else {
            plan_hard_farm_yield_v59_with_origin(
                &job.observation,
                &job.memory,
                &job.hazard_memory,
                job.settled_stop_tick,
                job.escape_origin.as_ref(),
                &mut rng,
                &mut work,
            )
        }
        .unwrap();
        let TeleporterRecoveryChoice::Ordinary(direct) = select_teleporter_recovery_v50(
            &job.observation,
            &job.hazard_memory,
            job.settled_stop_tick,
            &job.memory,
            job.escape_origin.as_ref(),
            direct,
            &mut work,
        ) else {
            panic!("the no-target fixture must retain its ordinary fallback")
        };
        assert!(planner.try_submit(job));
        let report = planner.pump_for_test(1);
        assert_eq!((report.delivered, report.failed), (1, 0));
        assert_eq!(report.work, vec![work.spent()]);
        let queued = result.recv().unwrap();
        assert_eq!(queued.ordinary().control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}
