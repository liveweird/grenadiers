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
