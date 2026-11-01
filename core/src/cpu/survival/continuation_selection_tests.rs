use crate::cpu::WorkBudget;
use crate::cpu::forecast::ObservedHazardMemory;
use crate::cpu::policy::{
    StrategicIntent, StrategicObjectiveKind, StrategicPhase, StrategicSwitchReason,
    StrategicTarget, UtilityMemory,
};
use crate::cpu::routing::NominalLeg;
use crate::game::{CellId, Direction, Position, Tick};

use super::WORK_ALLOWANCE;
use super::continuation_scene_fixture::{no_lair_scene, scene};
use super::continuation_selection::{OriginRecoveryChoice, plan_lairs};
use super::{continuation_recovery_memory, continuation_recovery_stop};

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
fn no_lair_allocator_certifies_both_mirrored_continuations() {
    let scene = no_lair_scene();
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
        .expect("no-lair shared allocator result");
        let OriginRecoveryChoice::Continuation(certificate) = choice else {
            panic!("no-lair continuation must fit its half-reservation")
        };
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
    let memory = strategic_memory(scene.key);
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
    let mut predecessor = super::origin_planner::plan_lairs(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut WorkBudget::new(20_000),
    )
    .expect("legacy origin recovery remains available");
    predecessor.survival.control.next_memory = continuation_recovery_memory(&memory);
    assert_eq!(plan.survival, predecessor.survival);
    assert_eq!(plan.contact, predecessor.contact);
    assert!(plan.survival.lease.is_some());
    assert!(work.spent() <= 20_000);
}

#[test]
fn no_lair_reduced_attempt_preserves_disabled_predecessor_fallback() {
    let scene = no_lair_scene();
    let memory = strategic_memory(scene.key);
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
    .expect("reserved fallback should retain disabled origin recovery");
    let OriginRecoveryChoice::Pulse(plan) = choice else {
        panic!("reduced attempt must use the disabled predecessor pulse")
    };
    let mut predecessor = super::origin_planner::plan(
        &scene.basis,
        &history,
        Tick(196),
        &memory,
        &scene.origin,
        &mut WorkBudget::new(20_000),
    )
    .expect("disabled origin recovery remains available");
    predecessor.control.next_memory = continuation_recovery_memory(&memory);
    assert_eq!(plan.survival, predecessor);
    assert!(plan.contact.is_none());
    assert!(work.spent() <= 20_000);
}

#[test]
fn continuation_recovery_clears_routes_and_preserves_strategic_progress() {
    let scene = scene();
    let memory = strategic_memory(scene.key);
    let recovered = continuation_recovery_memory(&memory);
    assert!(recovered.route().iter().all(Option::is_none));
    assert_eq!(recovered.planted_origin(), None);
    assert_eq!(recovered.refuge(), None);
    assert_eq!(recovered.strategic_intent(), memory.strategic_intent());
    let stopped = continuation_recovery_stop(&memory);
    assert_eq!(stopped.survival.control.next_memory, recovered);
    assert_eq!(
        stopped.survival.control.intent.movement,
        crate::cpu::MovementIntent::Stop
    );
    assert!(stopped.survival.lease.is_none());
    assert!(stopped.survival.execution.is_none());
    assert!(stopped.contact.is_none());
}

fn strategic_memory(key: crate::cpu::MemoryKey) -> UtilityMemory {
    UtilityMemory::from_route(key, &[CellId::new(1, 9), CellId::new(1, 10)], true)
        .with_strategic_intent(StrategicIntent {
            objective: StrategicObjectiveKind::Farm,
            target: StrategicTarget::Block {
                cell: CellId::new(3, 9),
            },
            approach_cell: Some(CellId::new(2, 9)),
            phase: StrategicPhase::Escape,
            last_switch_reason: StrategicSwitchReason::SafetyInterrupt,
            observation_tick: Tick(196),
            score: 2,
            committed_until: Tick(300),
            progress_deadline: Tick(376),
            last_distance: 1,
            last_cpu_position: Position { x: 1536, y: 9728 },
            deferred_until: None,
        })
}
