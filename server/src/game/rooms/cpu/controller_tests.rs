use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::sync_channel,
};

use grenadiers_core::cpu::{
    CpuRng, Intent, ManualIntent, MemoryKey, WorkBudget,
    policy::{ControlPlan, UtilityMemory, plan},
};
use grenadiers_core::game::{SeatId, Tick};

use super::{CpuController, Pending};
use crate::game::rooms::cpu::worker::{CpuPlanner, PlanResult};

fn controller() -> CpuController {
    let mut controller = CpuController::new(CpuPlanner::without_workers(), 7, [5; 32]);
    controller.reset_world(3);
    controller
}

fn inject(controller: &mut CpuController, decision: grenadiers_core::cpu::DecisionId) {
    let key = MemoryKey {
        world: decision.world,
        cpu_seat: SeatId::Two,
    };
    let mut rng = CpuRng::from_seed(key, [9; 32]);
    let _ = rng.bounded_u32(2).unwrap();
    let memory = UtilityMemory::new(key);
    let (sender, receiver) = sync_channel(1);
    sender
        .send(PlanResult {
            plan: ControlPlan::stop(memory),
            rng,
        })
        .unwrap();
    controller.pending = Some(Pending {
        decision,
        receiver,
        live: Arc::new(AtomicBool::new(true)),
    });
    controller.request_serial = decision.checked_request_serial;
}

fn decision(controller: &CpuController) -> grenadiers_core::cpu::DecisionId {
    grenadiers_core::cpu::DecisionId {
        world: controller.world.unwrap(),
        command_epoch: 4,
        observation_tick: Tick(20),
        checked_request_serial: 1,
    }
}

#[test]
fn stale_and_wrong_world_results_do_not_commit_prospective_rng() {
    let mut stale = controller();
    let stale_decision = decision(&stale);
    inject(&mut stale, stale_decision);
    assert!(stale.poll(Tick(27), 4).is_none());
    assert_eq!(stale.rng.as_ref().unwrap().draws(), 0);

    let mut wrong = controller();
    let mut wrong_decision = decision(&wrong);
    wrong_decision.world.room_instance += 1;
    inject(&mut wrong, wrong_decision);
    assert!(wrong.poll(Tick(20), 4).is_none());
    assert_eq!(wrong.rng.as_ref().unwrap().draws(), 0);
}

#[test]
fn duplicate_result_is_discarded_after_first_admission() {
    let mut controller = controller();
    let current = decision(&controller);
    inject(&mut controller, current);
    assert!(controller.poll(Tick(20), 4).is_some());
    inject(&mut controller, current);
    assert!(controller.poll(Tick(20), 4).is_none());
}

#[test]
fn admission_age_boundary_is_six_physical_ticks() {
    let mut controller = controller();
    let current = decision(&controller);
    inject(&mut controller, current);
    assert!(controller.poll(Tick(26), 4).is_some());
}

#[test]
fn expired_empty_request_is_canceled_and_does_not_block_retry() {
    let mut controller = controller();
    let (sender, receiver) = sync_channel(1);
    let live = Arc::new(AtomicBool::new(true));
    controller.pending = Some(Pending {
        decision: decision(&controller),
        receiver,
        live: Arc::clone(&live),
    });
    controller.request_serial = 1;

    assert!(controller.poll(Tick(27), 4).is_none());
    assert!(!live.load(Ordering::Acquire));
    assert!(!controller.has_pending());
    assert!(controller.submit(crate::game::rooms::cpu::tests::observation(), 4));
    drop(sender);
}

#[test]
fn rejected_bomb_clears_prospective_escape_but_commits_rng() {
    let mut controller = controller();
    let observation = crate::game::rooms::cpu::tests::observation();
    let key = key_for(&controller);
    let mut rng = CpuRng::from_seed(key, [4; 32]);
    let planned = plan(
        &observation,
        &UtilityMemory::new(key),
        &mut rng,
        &mut WorkBudget::new(256),
    )
    .unwrap();
    assert_eq!(planned.intent.manual, Some(ManualIntent::Bomb));
    let accepted = super::AcceptedPlan {
        decision: decision(&controller),
        intent: planned.intent,
        hold_ticks: planned.hold_ticks,
        next_memory: planned.next_memory,
        next_rng: rng.clone(),
    };

    controller.commit(
        accepted,
        Intent {
            movement: planned.intent.movement,
            manual: None,
        },
    );

    assert!(!controller.is_escaping());
    assert!(controller.memory.unwrap().planted_origin().is_none());
    assert_eq!(controller.rng.as_ref().unwrap().draws(), rng.draws());
}

#[test]
fn repeated_observation_tick_is_considered_only_once() {
    let mut controller = controller();
    let observation = crate::game::rooms::cpu::tests::observation();
    assert!(controller.accept_observation(&observation));
    assert!(!controller.accept_observation(&observation));
}

fn key_for(controller: &CpuController) -> MemoryKey {
    MemoryKey {
        world: controller.world.unwrap(),
        cpu_seat: SeatId::Two,
    }
}
