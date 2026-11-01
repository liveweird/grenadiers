use crate::cpu::WorkBudget;
use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::NominalLeg;
use crate::game::{Direction, Tick};

use super::WORK_ALLOWANCE;
use super::continuation_scene_fixture::scene;
use super::continuation_selection::{OriginRecoveryChoice, plan_lairs};

#[test]
fn shared_allocator_certifies_both_mirrored_continuations() {
    let scene = scene();
    for (basis, origin, key, lateral) in [
        (&scene.basis, &scene.origin, scene.key, Direction::East),
        (
            &scene.rival_basis,
            &scene.rival_origin,
            scene.rival_key,
            Direction::West,
        ),
    ] {
        let mut work = WorkBudget::new(WORK_ALLOWANCE);
        let choice = plan_lairs(
            basis,
            &ObservedHazardMemory::new(key),
            Tick(196),
            &UtilityMemory::new(key),
            origin,
            &mut work,
        )
        .expect("shared allocator result");
        let OriginRecoveryChoice::Continuation(certificate) = choice else {
            panic!("complete continuation proof must fit its actual half-reservation")
        };
        eprintln!("selector {:?} work={}", lateral, work.spent());
        assert_eq!(work.spent(), 15_995);
        assert!(work.spent() <= WORK_ALLOWANCE / 2);
        assert_eq!(
            certificate.legs_for_test(),
            &[
                NominalLeg::Move {
                    direction: Direction::South,
                    updates: 40,
                },
                NominalLeg::Stop { updates: 3 },
                NominalLeg::Move {
                    direction: lateral,
                    updates: 17,
                },
            ]
        );
    }
}

#[test]
fn reduced_attempt_preserves_same_forecast_legacy_fallback() {
    let scene = scene();
    let memory = UtilityMemory::new(scene.key);
    let history = ObservedHazardMemory::new(scene.key);
    let mut work = WorkBudget::new(20_000);
    let choice = plan_lairs(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut work,
    )
    .expect("reserved fallback should retain the old origin pulse");
    let OriginRecoveryChoice::Pulse(plan) = choice else {
        panic!("reduced attempt must use the predecessor pulse")
    };
    eprintln!("reduced selector fallback work={}", work.spent());
    assert_eq!(work.spent(), 9_895);
    let predecessor = super::origin_planner::plan_lairs(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut WorkBudget::new(20_000),
    )
    .expect("legacy origin recovery remains available");
    assert_eq!(plan.survival, predecessor.survival);
    assert_eq!(plan.contact, predecessor.contact);
    assert!(plan.survival.lease.is_some());
    assert!(work.spent() <= 20_000);
}
