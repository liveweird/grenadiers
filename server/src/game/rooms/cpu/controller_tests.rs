use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::sync_channel,
};

use grenadiers_core::cpu::{
    CpuRng, Intent, ManualIntent, MemoryKey, WorkBudget,
    forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast},
    policy::{
        ControlPlan, StrategicIntent, StrategicObjectiveKind, StrategicPhase,
        StrategicSwitchReason, StrategicTarget, UtilityMemory, plan,
    },
    routing::{PulseRequest, RouteMode, certify_pulse},
    survival::{PulseStatus, SurvivalPlan},
};
use grenadiers_core::game::{CellId, Direction, Position, SeatId, Tick};

use super::test_support::{key_for, observation_with_pose};
use super::{CpuController, Pending};
use crate::game::rooms::cpu::worker::{CpuPlanner, PlanResult};

fn controller() -> CpuController {
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [5; 32],
        grenadiers_core::room::CpuProfile::CURRENT,
    );
    controller.reset_world(3);
    controller
}

fn test_strategy() -> StrategicIntent {
    StrategicIntent {
        objective: StrategicObjectiveKind::Collect,
        target: StrategicTarget::Pickup {
            cell: CellId::new(5, 7),
            kind: grenadiers_core::game::PickupKind::GoldenFlame,
        },
        approach_cell: Some(CellId::new(5, 7)),
        phase: StrategicPhase::Approach,
        last_switch_reason: StrategicSwitchReason::Initial,
        observation_tick: Tick(20),
        score: 6,
        committed_until: Tick(44),
        progress_deadline: Tick(56),
        last_distance: 2,
        last_cpu_position: grenadiers_core::game::Position::center(CellId::new(7, 7)),
        deferred_until: None,
    }
}

fn inject(controller: &mut CpuController, decision: grenadiers_core::cpu::DecisionId) {
    let key = MemoryKey {
        world: decision.world,
        cpu_seat: SeatId::Two,
    };
    let mut rng = CpuRng::from_seed(key, [9; 32]);
    let _ = rng.bounded_u32(2).unwrap();
    let memory = UtilityMemory::new(key);
    let basis = crate::game::rooms::cpu::tests::observation();
    controller.latest_observation = Some(basis.clone());
    controller.settled_stop_tick = Some(basis.parts().tick);
    inject_plan(
        controller,
        decision,
        SurvivalPlan {
            control: ControlPlan::stop(memory),
            lease: None,
            execution: None,
        },
        basis,
        rng,
    );
}

fn inject_plan(
    controller: &mut CpuController,
    decision: grenadiers_core::cpu::DecisionId,
    plan: SurvivalPlan,
    basis: grenadiers_core::cpu::Observation,
    rng: CpuRng,
) {
    let (sender, receiver) = sync_channel(1);
    sender
        .send(PlanResult {
            outcome: super::super::worker::WorkerPlan::Ordinary {
                plan: Box::new(plan),
                contact: None,
            },
            basis,
            rng,
            escape_origin: None,
        })
        .unwrap();
    controller.pending = Some(Pending {
        decision,
        receiver,
        live: Arc::new(AtomicBool::new(true)),
        completed: None,
        refuge_handoff: None,
        continuation: None,
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
    let strategy = test_strategy();
    stale.memory = Some(stale.memory.unwrap().with_strategic_intent(strategy));
    let stale_decision = decision(&stale);
    inject(&mut stale, stale_decision);
    assert!(stale.poll(Tick(27), 4).is_none());
    assert_eq!(stale.rng.as_ref().unwrap().draws(), 0);
    assert_eq!(stale.memory.unwrap().strategic_intent(), Some(strategy));

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
fn fresh_world_change_rejects_even_a_non_lease_result() {
    let mut controller = controller();
    let basis = crate::game::rooms::cpu::tests::observation();
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(Tick(20));
    let current = decision(&controller);
    inject(&mut controller, current);
    let mut changed_parts = basis.into_parts();
    changed_parts.tick = Tick(21);
    changed_parts
        .arena
        .blocks
        .push(grenadiers_core::game::CellId::new(5, 5));
    let changed = grenadiers_core::cpu::Observation::new(changed_parts).unwrap();
    assert!(controller.accept_observation(&changed));

    assert!(controller.poll(Tick(21), 4).is_none());
    assert_eq!(controller.rng.as_ref().unwrap().draws(), 0);
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
        completed: None,
        refuge_handoff: None,
        continuation: None,
    });
    controller.request_serial = 1;

    assert!(controller.poll(Tick(27), 4).is_none());
    assert!(!live.load(Ordering::Acquire));
    assert!(!controller.has_pending());
    let before = crate::game::rooms::cpu::tests::observation();
    assert!(controller.accept_observation(&before));
    controller.on_stop_admitted(Tick(20));
    let mut stopped_parts = before.into_parts();
    stopped_parts.tick = Tick(21);
    let stopped = grenadiers_core::cpu::Observation::new(stopped_parts).unwrap();
    assert!(controller.accept_observation(&stopped));
    assert!(controller.submit(stopped, 4));
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
        committed_pulse: None,
        committed_route: None,
        contact: None,
        certified_manual: None,
        escape_origin: false,
        ordinary_navigation: false,
        refuge_handoff: false,
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

#[test]
fn stop_confirmation_waits_for_the_first_post_admission_snapshot() {
    let mut controller = controller();
    let initial = observation_at(Tick(20), 37);
    assert!(controller.accept_observation(&initial));
    controller.on_stop_admitted(Tick(22));

    let too_early = observation_at(Tick(21), 37);
    assert!(controller.accept_observation(&too_early));
    assert!(!controller.submit(too_early, 4));

    let settled = observation_at(Tick(23), 37);
    assert!(controller.accept_observation(&settled));
    assert!(controller.submit(settled, 4));
}

#[test]
fn post_stop_snapshot_confirms_its_new_non_center_pose() {
    let mut controller = controller();
    let initial = observation_at(Tick(20), 37);
    assert!(controller.accept_observation(&initial));
    controller.on_stop_admitted(Tick(20));
    assert!(controller.accept_observation(&observation_at(Tick(21), 89)));
    assert!(!controller.needs_stop_confirmation());

    controller.on_movement_admitted();

    assert!(controller.needs_stop_confirmation());
}

#[test]
fn admitted_long_pulse_waits_for_its_scheduled_stop_update() {
    let mut controller = controller();
    let basis = crate::game::rooms::cpu::tests::observation();
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let key = key_for(&controller);
    let forecast = forecast(
        &basis,
        &ObservedHazardMemory::new(key),
        None,
        ForecastLimits::default(),
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let lease = certify_pulse(
        &basis,
        &forecast,
        PulseRequest {
            movement: grenadiers_core::cpu::MovementIntent::Direction(Direction::West),
            hold_ticks: 12,
            start_window: TickWindow {
                start: Tick(21),
                end: Tick(22),
            },
            stopped_safe_until: Tick(50),
            mode: RouteMode::CertifiedOnly,
            settled_stop_tick: Tick(20),
        },
        &mut WorkBudget::new(65_536),
    )
    .unwrap();
    let final_pose = lease.pose_after(12);
    let intent = Intent::moving(Direction::West);
    let control = ControlPlan {
        intent,
        hold_ticks: 12,
        next_memory: UtilityMemory::new(key),
    };
    let pulse_decision = decision(&controller);
    inject_plan(
        &mut controller,
        pulse_decision,
        SurvivalPlan {
            control,
            lease: Some(lease),
            execution: None,
        },
        basis.clone(),
        CpuRng::from_seed(key, [8; 32]),
    );
    let accepted = controller.poll(Tick(20), 4).unwrap();
    controller.on_movement_admitted();
    controller.commit(accepted, intent);
    assert_eq!(
        controller.validate_pulse(&basis, Tick(20)),
        Some(PulseStatus::Continue)
    );

    let waiting = observation_with_pose(Tick(32), final_pose.min(), final_pose.heading_mask());
    assert!(controller.accept_observation(&waiting));
    assert_eq!(
        controller.validate_pulse(&waiting, Tick(32)),
        Some(PulseStatus::Wait)
    );
    let completed = observation_with_pose(Tick(33), final_pose.min(), final_pose.heading_mask());
    assert!(controller.accept_observation(&completed));
    assert_eq!(
        controller.validate_pulse(&completed, Tick(33)),
        Some(PulseStatus::Completed)
    );
}
fn observation_at(tick: Tick, x_offset: i32) -> grenadiers_core::cpu::Observation {
    let mut parts = crate::game::rooms::cpu::tests::observation().into_parts();
    parts.tick = tick;
    let grenadiers_core::cpu::ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()]
    else {
        unreachable!()
    };
    cpu.position.x += x_offset;
    grenadiers_core::cpu::Observation::new(parts).unwrap()
}

#[path = "controller_pulse_admission_tests.rs"]
mod pulse_admission_tests;
#[path = "controller_pulse_intent_tests.rs"]
mod pulse_intent_tests;
