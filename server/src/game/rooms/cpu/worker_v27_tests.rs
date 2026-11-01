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
    let planned = result.recv().unwrap().plan;
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
    assert_eq!(queued.plan.control, direct.survival.control);
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
    assert_eq!(queued.plan.control, direct.survival.control);
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
        assert_eq!(queued.plan.control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}

#[test]
fn queued_worker_dispatches_refuge_profiles_with_identical_core_control_rng_and_work() {
    use grenadiers_core::cpu::{
        WorkBudget,
        survival::{plan_refuge_guidance_v33_with_origin, plan_refuge_guidance_v34_with_origin},
    };
    for policy in [CpuPolicy::UtilityV33, CpuPolicy::UtilityV34] {
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
        assert_eq!(queued.plan.control, direct.survival.control);
        assert_eq!(queued.rng.draws(), rng.draws());
    }
}
