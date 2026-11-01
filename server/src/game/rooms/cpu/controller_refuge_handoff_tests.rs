use grenadiers_core::room::CpuProfile;

use super::CpuController;
use super::refuge_handoff_test_support::{EPOCH, queued_refuge, queued_refuge_for};

#[test]
fn archived_hard_profile_cannot_enter_otherwise_eligible_refuge_replacement_flow() {
    for retained in [CpuProfile::HARD_V29, CpuProfile::HARD_V34] {
        let mut fixture = queued_refuge();
        fixture.controller.cancel();
        fixture.controller.refuge_attempt = None;
        fixture.controller.profile = retained;
        let memory = fixture.controller.memory;
        let rng_draws = fixture.controller.rng_draws();
        assert!(
            !fixture
                .controller
                .submit_refuge_handoff(fixture.publication(), EPOCH)
        );
        assert!(!fixture.controller.has_pending());
        assert!(fixture.controller.has_committed_route());
        assert_eq!(fixture.controller.memory, memory);
        assert_eq!(fixture.controller.rng_draws(), rng_draws);
        fixture.controller.profile = CpuProfile::HARD_V35;
        assert!(
            fixture
                .controller
                .submit_refuge_handoff(fixture.publication(), EPOCH)
        );
    }
}

#[test]
fn delayed_worker_keeps_the_installed_hold_and_live_state_until_admission() {
    let mut fixture = queued_refuge();
    let memory = fixture.controller.memory;
    let rng_draws = fixture.controller.rng_draws();
    assert!(fixture.controller.has_committed_route());
    assert!(fixture.controller.has_pending());
    assert!(
        fixture
            .controller
            .poll(fixture.state.tick(), EPOCH)
            .is_none()
    );

    fixture.advance_publication();
    assert!(fixture.controller.has_committed_route());
    assert_eq!(fixture.controller.memory, memory);
    assert_eq!(fixture.controller.rng_draws(), rng_draws);
    assert_eq!(fixture.planner.pump_for_test(1).delivered, 1);

    let mut accepted = None;
    for _ in 0..=grenadiers_core::cpu::survival::MAX_PLAN_AGE_TICKS {
        accepted = fixture.controller.poll(fixture.state.tick(), EPOCH);
        if accepted.is_some() {
            break;
        }
        fixture.advance_publication();
    }
    let accepted = accepted.expect("fresh replacement reaches exact admission");
    assert!(accepted.is_refuge_handoff());
    assert!(fixture.controller.has_committed_route());
    assert_eq!(fixture.controller.memory, memory);
    assert_eq!(fixture.controller.rng_draws(), rng_draws);
}

#[test]
fn hard37_queues_and_admits_a_real_refuge_replacement() {
    let mut fixture = queued_refuge_for(CpuProfile::HARD_V37);
    assert!(fixture.controller.has_pending());
    assert_eq!(fixture.planner.pump_for_test(1).delivered, 1);

    let mut accepted = None;
    for _ in 0..=grenadiers_core::cpu::survival::MAX_PLAN_AGE_TICKS {
        accepted = fixture.controller.poll(fixture.state.tick(), EPOCH);
        if accepted.is_some() {
            break;
        }
        fixture.advance_publication();
    }
    assert!(
        accepted.is_some_and(|plan| plan.is_refuge_handoff()),
        "Hard37 worker result did not reach ordinary handoff admission"
    );
}

#[test]
fn hard39_queues_and_admits_a_real_refuge_replacement() {
    let mut fixture = queued_refuge_for(CpuProfile::HARD_V39);
    assert!(fixture.controller.has_pending());
    assert_eq!(fixture.planner.pump_for_test(1).delivered, 1);

    let mut accepted = None;
    for _ in 0..=grenadiers_core::cpu::survival::MAX_PLAN_AGE_TICKS {
        accepted = fixture.controller.poll(fixture.state.tick(), EPOCH);
        if accepted.is_some() {
            break;
        }
        fixture.advance_publication();
    }
    assert!(
        accepted.is_some_and(|plan| plan.is_refuge_handoff()),
        "Hard39 worker result did not reach ordinary handoff admission"
    );
}

#[test]
fn cancellation_and_epoch_staleness_preserve_the_old_route_memory_and_rng() {
    for cancel in [true, false] {
        let mut fixture = queued_refuge();
        let memory = fixture.controller.memory;
        let rng_draws = fixture.controller.rng_draws();
        if cancel {
            fixture.controller.cancel();
            let publication = fixture.advance_publication();
            assert!(
                !fixture
                    .controller
                    .submit_refuge_handoff_for_test(publication, EPOCH)
            );
        } else {
            assert!(
                fixture
                    .controller
                    .poll(fixture.state.tick(), EPOCH + 1)
                    .is_none()
            );
        }
        assert!(fixture.controller.has_committed_route());
        assert!(!fixture.controller.has_pending());
        assert_eq!(fixture.controller.memory, memory);
        assert_eq!(fixture.controller.rng_draws(), rng_draws);
    }
}

#[test]
fn nonplant_worker_result_is_rejected_without_replacing_old_authority() {
    let mut fixture = queued_refuge();
    let memory = fixture.controller.memory;
    let rng_draws = fixture.controller.rng_draws();
    assert_eq!(fixture.planner.pump_for_test(1).delivered, 1);
    assert!(
        fixture
            .controller
            .poll(fixture.state.tick(), EPOCH)
            .is_none()
    );
    let result = fixture
        .controller
        .pending
        .as_mut()
        .unwrap()
        .completed
        .as_mut()
        .unwrap();
    let route = result.plan.execution.as_ref().unwrap().route().clone();
    result.plan.execution = Some(grenadiers_core::cpu::survival::ExecutionCertificate::Route(
        Box::new(route),
    ));
    for _ in 0..=grenadiers_core::cpu::survival::MAX_PLAN_AGE_TICKS {
        assert!(
            fixture
                .controller
                .poll(fixture.state.tick(), EPOCH)
                .is_none()
        );
        if !fixture.controller.has_pending() {
            break;
        }
        fixture.advance_publication();
    }
    assert!(!fixture.controller.has_pending());
    assert!(fixture.controller.has_committed_route());
    assert_eq!(fixture.controller.memory, memory);
    assert_eq!(fixture.controller.rng_draws(), rng_draws);
    assert_eq!(
        fixture.controller.admission_trace_for_test().refuge_handoff,
        1
    );
}

#[test]
fn full_shared_queue_leaves_the_old_hold_and_live_state_untouched() {
    let mut fixture = queued_refuge();
    fixture.controller.cancel();
    fixture.controller.refuge_attempt = None;
    let memory = fixture.controller.memory;
    let rng_draws = fixture.controller.rng_draws();
    let ordinary = crate::game::rooms::cpu::tests::observation();
    let mut fillers = Vec::new();
    for seed in 0..31 {
        let mut filler = CpuController::new(
            fixture.planner.clone(),
            ordinary.parts().recipient,
            ordinary.parts().world.room_instance,
            [seed; 32],
            CpuProfile::CURRENT,
        );
        assert!(filler.accept_observation(&ordinary));
        filler.settled_stop_tick = Some(ordinary.parts().tick);
        assert!(filler.submit(ordinary.clone(), EPOCH));
        fillers.push(filler);
    }
    assert!(
        !fixture
            .controller
            .submit_refuge_handoff_for_test(fixture.publication(), EPOCH)
    );
    assert!(fixture.controller.has_committed_route());
    assert!(!fixture.controller.has_pending());
    assert_eq!(fixture.controller.memory, memory);
    assert_eq!(fixture.controller.rng_draws(), rng_draws);
    assert_eq!(fillers.len(), 31);
}
