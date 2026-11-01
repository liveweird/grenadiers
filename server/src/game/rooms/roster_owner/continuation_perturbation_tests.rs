use tokio::time::Instant;

use super::{
    continuation_owner_support::{CPUS, driver},
    driver::Driver,
};
use crate::game::rooms::cpu::CpuPlanner;

fn admitted() -> (Driver, CpuPlanner) {
    let (mut owner, planner) = driver(false);
    owner.tick(Instant::now()).unwrap();
    for _ in 0..300 {
        planner.pump_for_test(usize::MAX);
        owner.tick(Instant::now()).unwrap();
        if owner
            .cpus
            .iter()
            .all(|cpu| cpu.has_continuation_authority_for_test())
        {
            return (owner, planner);
        }
    }
    panic!("continuation authority was not admitted");
}

fn positions(owner: &Driver) -> [grenadiers_core::game::Position; 2] {
    let state = owner.runtime.state_for_test().unwrap();
    CPUS.map(|seat| state.actor(seat).unwrap().position())
}

#[test]
fn actual_early_publication_after_one_or_two_reports_revokes_and_stops() {
    for reports in [1, 2] {
        let (mut owner, _planner) = admitted();
        while owner.runtime.pending_report_count_for_test() < reports {
            owner.tick(Instant::now()).unwrap();
        }
        assert_eq!(owner.runtime.pending_report_count_for_test(), reports);
        let before = positions(&owner);
        let notice = owner.runtime.current_notice().unwrap();
        owner.publish(notice);
        assert_eq!(owner.runtime.pending_report_count_for_test(), 0);
        assert!(
            owner
                .cpus
                .iter()
                .all(|cpu| !cpu.has_continuation_authority_for_test())
        );

        owner.tick(Instant::now()).unwrap();
        assert_eq!(positions(&owner), before);
    }
}

#[test]
fn delayed_worker_replays_actual_b_plus_three_before_exact_admission() {
    let (mut owner, planner) = driver(false);
    owner.tick(Instant::now()).unwrap();
    let basis = loop {
        planner.pump_for_test(usize::MAX);
        owner.tick(Instant::now()).unwrap();
        if owner
            .cpus
            .iter()
            .all(|cpu| cpu.recovery_state().0 && cpu.has_pending())
        {
            break owner.runtime.lifecycle().tick();
        }
    };
    for _ in 0..3 {
        owner.tick(Instant::now()).unwrap();
    }
    assert_eq!(owner.runtime.lifecycle().tick().0, basis.0 + 3);
    assert!(owner.cpus.iter().all(|cpu| cpu.has_pending()));
    assert_eq!(planner.pump_for_test(usize::MAX).failed, 0);
    for _ in 0..4 {
        owner.tick(Instant::now()).unwrap();
    }
    assert!(
        owner
            .cpus
            .iter()
            .all(|cpu| cpu.continuation_trace_for_test().admission_tick.is_some())
    );
}

#[test]
fn missing_renewal_fences_before_the_first_unauthorized_update() {
    let (mut owner, _planner) = admitted();
    let admission = owner.cpus[0]
        .continuation_trace_for_test()
        .admission_tick
        .unwrap();
    let renewal = grenadiers_core::game::Tick(admission.0 + 3);
    while owner.runtime.lifecycle().tick() < renewal {
        owner.service_cpu().unwrap();
        let notice = owner.runtime.settle_boundary([None; 4]).unwrap();
        if let Some(notice) = notice
            && owner.runtime.lifecycle().tick() < renewal
        {
            owner.publish(notice);
        }
    }
    let before = positions(&owner);
    owner.service_cpu().unwrap();
    owner.runtime.settle_boundary([None; 4]).unwrap();
    assert_eq!(positions(&owner), before);
    assert!(!owner.cpus[0].has_continuation_authority_for_test());
}

#[test]
fn rejected_command_and_rejected_stop_still_clear_held_input() {
    let (mut owner, _planner) = admitted();
    let before = positions(&owner);
    owner.next_cpu = 0;
    owner.continuation_rejections = [true; 2];
    owner.tick(Instant::now()).unwrap();

    assert_eq!(positions(&owner)[0], before[0]);
    assert!(!owner.cpus[0].has_continuation_authority_for_test());
    assert!(owner.cpu_stationary[CPUS[0].index()]);
}

#[test]
fn changed_epoch_revokes_the_sealed_authority() {
    let (mut owner, _planner) = admitted();
    let tick = owner.runtime.lifecycle().tick();
    let wrong_epoch = owner.runtime.lifecycle().round_attempt() + 1;
    assert!(matches!(
        owner.cpus[0].service_continuation(tick, wrong_epoch),
        crate::game::rooms::cpu::ContinuationService::Invalidated
    ));
    assert!(!owner.cpus[0].has_continuation_authority_for_test());
}

#[test]
fn publication_sequence_overflow_is_unavailable_and_fences_authority() {
    let (mut owner, _planner) = admitted();
    owner.publication_sequence = Some(u64::MAX);
    let before = positions(&owner);
    let notice = owner.runtime.current_notice().unwrap();
    owner.publish(notice);
    assert_eq!(owner.publication_sequence, None);
    assert!(
        owner
            .cpus
            .iter()
            .all(|cpu| !cpu.has_continuation_authority_for_test())
    );
    owner.tick(Instant::now()).unwrap();
    assert_eq!(positions(&owner), before);
}
