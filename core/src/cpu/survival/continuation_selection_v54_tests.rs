use crate::cpu::{WorkBudget, forecast::ObservedHazardMemory, policy::UtilityMemory};
use crate::game::Tick;

use super::{
    continuation_scene_fixture::scene,
    continuation_selection::{OriginRecoveryChoice, plan_lairs_avoiding_pits, plan_lairs_v54},
};

#[test]
fn v54_prepares_once_then_reserves_optional_continuation_work() {
    let scene = scene();
    let history = ObservedHazardMemory::new(scene.key);
    let memory = UtilityMemory::new(scene.key);
    let mut legacy_work = WorkBudget::new(65_536);
    let legacy = plan_lairs_avoiding_pits(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut legacy_work,
    )
    .expect("legacy successful preparation");
    let mut work = WorkBudget::new(65_536);
    let choice = plan_lairs_v54(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut work,
    )
    .expect("full shared preparation must produce recovery authority");
    eprintln!("v54 full choice={} work={}", kind(&choice), work.spent());
    let (OriginRecoveryChoice::Continuation(legacy), OriginRecoveryChoice::Continuation(choice)) =
        (legacy, choice)
    else {
        panic!("both selectors must preserve the successful continuation")
    };
    assert_eq!(choice.legs_for_test(), legacy.legs_for_test());
    assert_eq!(work.spent(), legacy_work.spent());
    assert_eq!(work.spent(), 15_995);
}

#[test]
fn v54_optional_failure_leaves_reserved_pulse_fallback() {
    let scene = scene();
    let memory = UtilityMemory::new(scene.key);
    let history = ObservedHazardMemory::new(scene.key);
    let mut legacy_work = WorkBudget::new(20_000);
    let legacy = plan_lairs_avoiding_pits(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut legacy_work,
    )
    .expect("legacy reserved fallback");
    let mut work = WorkBudget::new(20_000);
    let choice = plan_lairs_v54(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut work,
    )
    .expect("reserved remainder must retain a recovery pulse");
    eprintln!(
        "v54 reserved choice={} work={}",
        kind(&choice),
        work.spent()
    );
    let OriginRecoveryChoice::Pulse(plan) = choice else {
        panic!("bounded optional proof must yield to the reserved fallback")
    };
    let OriginRecoveryChoice::Pulse(legacy) = legacy else {
        panic!("legacy selector must also yield to its fallback")
    };
    assert_eq!(plan.survival, legacy.survival);
    assert_eq!(plan.contact, legacy.contact);
    assert!(plan.survival.lease.is_some());
    assert_eq!(
        plan.survival.control.next_memory,
        super::continuation_recovery_memory(&memory)
    );
    assert_eq!(work.spent(), legacy_work.spent());
    assert_eq!(work.spent(), 9_895);
}

#[test]
fn v54_uses_remaining_allowance_after_parent_work() {
    let scene = scene();
    let history = ObservedHazardMemory::new(scene.key);
    let memory = UtilityMemory::new(scene.key);
    let mut legacy_work = WorkBudget::new(65_536);
    let legacy = plan_lairs_avoiding_pits(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut legacy_work,
    )
    .expect("legacy continuation");
    let mut parent = WorkBudget::new(66_770);
    parent.try_take(1_234).unwrap();
    let before = parent.spent();
    let selected = plan_lairs_v54(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut parent,
    )
    .expect("v54 continuation after prior parent work");
    let (OriginRecoveryChoice::Continuation(legacy), OriginRecoveryChoice::Continuation(selected)) =
        (legacy, selected)
    else {
        panic!("partly spent parent must preserve the successful choice")
    };
    assert_eq!(selected.legs_for_test(), legacy.legs_for_test());
    assert_eq!(parent.spent() - before, legacy_work.spent());
}

#[test]
fn v54_fails_closed_when_common_preparation_cannot_fit() {
    let scene = scene();
    for (limit, expected_spent) in [(0, 0), (7, 4), (5_000, 5_000)] {
        let mut work = WorkBudget::new(limit);
        let choice = plan_lairs_v54(
            &scene.basis,
            &ObservedHazardMemory::new(scene.key),
            Tick(196),
            &UtilityMemory::new(scene.key),
            &scene.origin,
            &mut work,
        );
        eprintln!(
            "v54 low limit={limit} work={} choice={}",
            work.spent(),
            choice.as_ref().map_or("none", kind)
        );
        assert!(choice.is_none());
        assert_eq!(work.spent(), expected_spent);
    }
}

#[test]
fn v54_rejects_a_foreign_origin_without_manufacturing_a_lease() {
    let scene = scene();
    let mut work = WorkBudget::new(65_536);
    let choice = plan_lairs_v54(
        &scene.basis,
        &ObservedHazardMemory::new(scene.key),
        Tick(196),
        &UtilityMemory::new(scene.key),
        &scene.rival_origin,
        &mut work,
    );
    assert!(choice.is_none());
    assert_eq!(work.spent(), 321);
}

fn kind(choice: &OriginRecoveryChoice) -> &'static str {
    match choice {
        OriginRecoveryChoice::Continuation(_) => "continuation",
        OriginRecoveryChoice::Pulse(_) => "pulse",
    }
}
