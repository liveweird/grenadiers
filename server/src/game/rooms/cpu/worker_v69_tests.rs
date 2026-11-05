use grenadiers_core::{
    cpu::{
        WorkBudget,
        survival::{
            TeleporterRecoveryChoice, WORK_ALLOWANCE, plan_hard_pickup_v66_with_origin,
            plan_hard_progress_v69_with_origin, select_teleporter_recovery_v50,
        },
    },
    room::CpuPolicy,
};

use super::{
    CpuPlanner, tests::spike_job, worker::WorkerPlan,
    worker_teleporter_tests::use_policy_observation,
};

#[test]
fn ordinary_hard69_uses_progress69_with_exact_plan_work_and_rng() {
    let planner = CpuPlanner::without_workers();
    let (mut job, result) = spike_job(CpuPolicy::UtilityV69);
    let (mut archived_job, archived_result) = spike_job(CpuPolicy::UtilityV67);
    use_policy_observation(&mut job);
    use_policy_observation(&mut archived_job);
    assert_eq!(job.observation, archived_job.observation);
    assert_eq!(job.memory, archived_job.memory);
    assert_eq!(job.hazard_memory, archived_job.hazard_memory);
    assert_eq!(job.settled_stop_tick, archived_job.settled_stop_tick);
    let mut rng = job.rng.clone();
    let mut archived_rng = archived_job.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let mut archived_work = WorkBudget::new(WORK_ALLOWANCE);
    let direct = plan_hard_progress_v69_with_origin(
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
    let archived = plan_hard_pickup_v66_with_origin(
        &job.observation,
        &job.memory,
        &job.hazard_memory,
        job.settled_stop_tick,
        job.escape_origin.as_ref(),
        &mut archived_rng,
        &mut archived_work,
    )
    .unwrap();
    let TeleporterRecoveryChoice::Ordinary(archived) = select_teleporter_recovery_v50(
        &job.observation,
        &job.hazard_memory,
        job.settled_stop_tick,
        &job.memory,
        job.escape_origin.as_ref(),
        archived,
        &mut archived_work,
    ) else {
        panic!("the unaffected archived planner retains its ordinary fallback")
    };

    assert_eq!(direct.survival, archived.survival);
    assert_eq!(direct.contact, archived.contact);
    assert_eq!(work.spent(), archived_work.spent());
    assert_eq!(rng.draws(), archived_rng.draws());

    assert!(job.refuge_handoff.is_none());
    assert!(archived_job.refuge_handoff.is_none());
    assert!(planner.try_submit(job));
    assert!(planner.try_submit(archived_job));
    let report = planner.pump_for_test(2);
    assert_eq!((report.delivered, report.failed), (2, 0));
    assert_eq!(report.work, vec![work.spent(), archived_work.spent()]);
    let queued = result.recv().unwrap();
    let archived_queued = archived_result.recv().unwrap();
    let WorkerPlan::Ordinary { plan, contact } = queued.outcome else {
        panic!("ordinary Hard69 dispatch must retain the typed result shape")
    };
    let WorkerPlan::Ordinary {
        plan: archived_plan,
        contact: archived_contact,
    } = archived_queued.outcome
    else {
        panic!("ordinary Hard67 dispatch must retain the typed result shape")
    };
    assert_eq!(*plan, direct.survival);
    assert_eq!(contact, direct.contact);
    assert_eq!(*archived_plan, direct.survival);
    assert_eq!(archived_contact, direct.contact);
    let mut queued_rng = queued.rng;
    let mut archived_queued_rng = archived_queued.rng;
    assert_eq!(
        (queued_rng.key(), queued_rng.draws()),
        (rng.key(), rng.draws())
    );
    for upper in [1, 2, 3, 17, 257, u32::MAX] {
        let expected = archived_rng.bounded_u32(upper);
        assert_eq!(rng.bounded_u32(upper), expected);
        assert_eq!(queued_rng.bounded_u32(upper), expected);
        assert_eq!(archived_queued_rng.bounded_u32(upper), expected);
    }
}
