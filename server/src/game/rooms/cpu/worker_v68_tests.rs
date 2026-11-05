use grenadiers_core::{
    cpu::{
        WorkBudget,
        survival::{
            TeleporterRecoveryChoice, WORK_ALLOWANCE, plan_contact_punch_v60_with_origin,
            select_teleporter_recovery_v50,
        },
    },
    room::CpuPolicy,
};

use super::{
    CpuPlanner, tests::spike_job, worker::WorkerPlan,
    worker_teleporter_tests::use_policy_observation,
};

#[test]
fn ordinary_easy68_exactly_matches_easy62_plan_contact_work_and_rng() {
    let planner = CpuPlanner::without_workers();
    let (mut job, result) = spike_job(CpuPolicy::UtilityV68);
    use_policy_observation(&mut job);
    let mut rng = job.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let direct = plan_contact_punch_v60_with_origin(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut rng,
        &mut work,
    )
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
        panic!("an observation without a target retains its ordinary fallback")
    };

    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!((report.delivered, report.failed), (1, 0));
    assert_eq!(report.work, vec![work.spent()]);
    let queued = result.recv().unwrap();
    let WorkerPlan::Ordinary { plan, contact } = queued.outcome else {
        panic!("ordinary Easy68 dispatch preserves the Easy62 result shape")
    };
    assert_eq!(*plan, direct.survival);
    assert_eq!(contact, direct.contact);
    let mut queued_rng = queued.rng;
    assert_eq!(
        (queued_rng.key(), queued_rng.draws()),
        (rng.key(), rng.draws())
    );
    for upper in [1, 2, 3, 17, 257, u32::MAX] {
        assert_eq!(queued_rng.bounded_u32(upper), rng.bounded_u32(upper));
    }
}
