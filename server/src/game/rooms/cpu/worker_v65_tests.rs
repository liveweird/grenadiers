use grenadiers_core::{
    cpu::{
        WorkBudget,
        survival::{WORK_ALLOWANCE, plan_hard_reactive_v65_with_origin},
    },
    room::CpuPolicy,
};

use super::{CpuPlanner, tests::spike_job, worker_teleporter_tests::teleporter_observation};

#[test]
fn queued_worker_dispatches_hard65_with_direct_failure_and_work_parity() {
    let planner = CpuPlanner::without_workers();
    let (mut job, result) = spike_job(CpuPolicy::UtilityV65);
    job.observation = teleporter_observation();
    job.hazard_memory = grenadiers_core::cpu::forecast::ObservedHazardMemory::new(job.memory.key());
    job.hazard_memory.observe(&job.observation).unwrap();
    let mut rng = job.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let direct = plan_hard_reactive_v65_with_origin(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut rng,
        &mut work,
    )
    .unwrap_err();

    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (0, 1));
    assert_eq!(report.work, vec![work.spent()]);
    assert!(report.work[0] <= WORK_ALLOWANCE);
    assert!(result.recv().is_err());
    assert_eq!(
        direct,
        grenadiers_core::cpu::policy::PolicyError::InvalidObservation
    );
}
