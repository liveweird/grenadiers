use grenadiers_core::cpu::Intent;
use grenadiers_core::cpu::policy::{ControlPlan, UtilityMemory};
use grenadiers_core::cpu::survival::SurvivalPlan;
use grenadiers_core::game::{SeatId, Tick};
use grenadiers_core::room::CpuProfile;

use super::{ContinuationService, CpuController};
use crate::game::rooms::cpu::CpuPlanner;

#[test]
fn ordinary_continuation_fallback_releases_clock_for_normal_admission() {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [44; 32],
        CpuProfile::CONTINUATION_V44_STANDARD,
    );
    let basis = crate::game::rooms::cpu::tests::spike_observation();
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let key = controller.memory.as_ref().unwrap().key();
    controller.inject_plan_result_for_test(
        basis.clone(),
        4,
        SurvivalPlan {
            control: ControlPlan::stop(UtilityMemory::new(key)),
            lease: None,
            execution: None,
        },
    );
    controller.mark_pending_continuation_for_test(9);

    assert!(matches!(
        controller.service_continuation(basis.parts().tick, 4),
        ContinuationService::Idle
    ));
    let accepted = controller.poll(basis.parts().tick, 4).unwrap();
    assert_eq!(accepted.intent, Intent::STOP);
}

#[test]
fn sequence_overflow_cancels_pending_continuation_attempt() {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [45; 32],
        CpuProfile::HARD_V45,
    );
    let basis = crate::game::rooms::cpu::tests::spike_observation();
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(Tick(20));
    let key = controller.memory.as_ref().unwrap().key();
    controller.inject_plan_result_for_test(
        basis,
        4,
        SurvivalPlan {
            control: ControlPlan::stop(UtilityMemory::new(key)),
            lease: None,
            execution: None,
        },
    );
    controller.mark_pending_continuation_for_test(u64::MAX);

    assert!(!controller.observe_continuation_publication(u64::MAX, None));
    assert!(!controller.has_pending());
}
