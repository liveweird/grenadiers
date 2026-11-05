use grenadiers_core::cpu::{ObservedActor, ObservedFlame, survival::PulseStatus};
use grenadiers_core::game::{CellId, Direction, PickupKind, SeatId, Tick};
use grenadiers_core::room::{CpuPolicy, CpuProfile};

use super::super::CpuController;
use crate::game::rooms::cpu::{
    CpuPlanner,
    tests::spike_job,
    worker::WorkerPlan,
    worker_teleporter_tests::{teleporter_observation, use_policy_observation},
};

const EPOCH: u64 = 4;

#[test]
fn completed_certified_pulse_waits_for_real_stop_then_pauses_once() {
    let (mut controller, basis, final_position, heading, completion) =
        admitted_pulse(CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    assert_eq!(controller.easy68_cadence_metrics_for_test().pause_armed, 1);
    if completion > basis.parts().tick.0 + 1 {
        let waiting = with_pose(&basis, completion - 1, final_position, heading);
        assert!(controller.accept_observation(&waiting));
        assert_ne!(
            controller.validate_pulse(&waiting, Tick(completion - 1)),
            Some(PulseStatus::Completed)
        );
    }
    let completed = with_pose(&basis, completion, final_position, heading);
    assert!(controller.accept_observation(&completed));
    assert_eq!(
        controller.validate_pulse(&completed, Tick(completion)),
        Some(PulseStatus::Completed)
    );
    controller.on_stop_admitted(Tick(completion));

    for tick in completion + 1..completion + 13 {
        let stopped = with_pose(&basis, tick, final_position, heading);
        assert!(controller.accept_observation(&stopped));
        assert!(!controller.submit(stopped, EPOCH));
    }
    let due = with_pose(&basis, completion + 13, final_position, heading);
    assert!(controller.accept_observation(&due));
    assert!(controller.submit(due, EPOCH));
    let metrics = controller.easy68_cadence_metrics_for_test();
    assert_eq!((metrics.pause_armed, metrics.pause_completed), (1, 1));
    assert_eq!(metrics.planning_deferred, 12);
    assert_eq!(metrics.planning_bypassed, 0);
    assert!(metrics.guard_work > 0);
}

#[test]
fn standard62_never_arms_from_the_same_admitted_pulse() {
    let (controller, _, _, _, _) = admitted_pulse(CpuProfile::RECOVERY_V62_STANDARD);
    assert_eq!(
        controller.easy68_cadence_metrics_for_test(),
        Default::default()
    );
}

#[test]
fn invalidation_epoch_danger_and_displacement_bypass_without_refreshing() {
    let (mut invalidated, basis, _, _, _) = admitted_pulse(CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    let displaced = with_pose(
        &basis,
        21,
        grenadiers_core::game::Position::center(CellId::new(3, 3)),
        Direction::East,
    );
    assert!(invalidated.accept_observation(&displaced));
    assert_eq!(
        invalidated.validate_pulse(&displaced, Tick(21)),
        Some(PulseStatus::Invalidated)
    );
    assert_eq!(
        invalidated
            .easy68_cadence_metrics_for_test()
            .planning_bypassed,
        1
    );

    let (mut epoch, _, _, _, _) = admitted_pulse(CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    epoch.validate_escape_epoch(EPOCH + 1);
    assert_eq!(epoch.easy68_cadence_metrics_for_test().planning_bypassed, 1);

    let (mut danger, basis, final_position, heading, next_tick) = ready_to_pause();
    let mut dangerous = with_pose(&basis, next_tick, final_position, heading).into_parts();
    dangerous.flames.push(ObservedFlame {
        cell: CellId::new(6, 6),
        remaining_ticks: 2,
    });
    let dangerous = basis.clone().map_parts(|parts| *parts = dangerous).unwrap();
    assert!(danger.accept_observation(&dangerous));
    assert!(danger.submit(dangerous, EPOCH));
    assert_eq!(
        danger.easy68_cadence_metrics_for_test().planning_bypassed,
        1
    );

    let (mut moved, basis, final_position, heading, next_tick) = ready_to_pause();
    let displaced = grenadiers_core::game::Position {
        x: final_position.x + 1,
        y: final_position.y,
    };
    let moved_observation = with_pose(&basis, next_tick, displaced, heading);
    assert!(moved.accept_observation(&moved_observation));
    assert!(moved.submit(moved_observation, EPOCH));
    assert_eq!(moved.easy68_cadence_metrics_for_test().planning_bypassed, 1);
}

fn ready_to_pause() -> (
    CpuController,
    grenadiers_core::cpu::Observation,
    grenadiers_core::game::Position,
    Direction,
    u64,
) {
    let (mut controller, basis, final_position, heading, completion) =
        admitted_pulse(CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    let completed = with_pose(&basis, completion, final_position, heading);
    assert!(controller.accept_observation(&completed));
    assert_eq!(
        controller.validate_pulse(&completed, Tick(completion)),
        Some(PulseStatus::Completed)
    );
    controller.on_stop_admitted(Tick(completion));
    let stopped_tick = completion + 1;
    let stopped = with_pose(&basis, stopped_tick, final_position, heading);
    assert!(controller.accept_observation(&stopped));
    assert!(!controller.submit(stopped, EPOCH));
    (controller, basis, final_position, heading, stopped_tick + 1)
}

fn admitted_pulse(
    profile: CpuProfile,
) -> (
    CpuController,
    grenadiers_core::cpu::Observation,
    grenadiers_core::game::Position,
    Direction,
    u64,
) {
    let planner = CpuPlanner::without_workers();
    let (mut job, result) = spike_job(CpuPolicy::UtilityV68);
    job.observation = teleporter_observation()
        .map_parts(|parts| {
            parts.pickups.clear();
            parts.pickups.push(grenadiers_core::cpu::ObservedPickup {
                cell: CellId::new(5, 7),
                kind: PickupKind::BombCapacity,
            });
        })
        .unwrap();
    use_policy_observation(&mut job);
    let basis = job.observation.clone();
    assert!(planner.try_submit(job));
    assert_eq!(planner.pump_for_test(1).delivered, 1);
    let WorkerPlan::Ordinary { plan, .. } = result.recv().unwrap().outcome else {
        panic!("pickup fixture must retain the ordinary planner")
    };
    let plan = *plan;
    let mut controller = CpuController::new(
        CpuPlanner::without_workers(),
        SeatId::Two,
        7,
        [68; 32],
        profile,
    );
    assert!(controller.accept_observation(&basis));
    controller.settled_stop_tick = Some(basis.parts().tick);
    let lease = plan
        .lease
        .as_ref()
        .expect("generated pickup plan has a pulse");
    let hold_ticks = plan.control.hold_ticks;
    let intent = plan.control.intent;
    let pose = lease.pose_after(hold_ticks);
    let heading = heading(pose.heading_mask());
    controller.inject_plan_result_for_test(basis.clone(), EPOCH, plan);
    let accepted = controller.poll(basis.parts().tick, EPOCH).unwrap();
    let pulse = accepted.committed_pulse.as_ref().unwrap();
    let completion = pulse.first_execution().0 + u64::from(pulse.lease().hold_ticks());
    assert!(!accepted.ordinary_navigation);
    controller.on_movement_admitted();
    controller.commit(accepted, intent);
    (controller, basis, pose.min(), heading, completion)
}

fn with_pose(
    basis: &grenadiers_core::cpu::Observation,
    tick: u64,
    position: grenadiers_core::game::Position,
    heading: Direction,
) -> grenadiers_core::cpu::Observation {
    basis
        .clone()
        .map_parts(|parts| {
            parts.tick = Tick(tick);
            parts.rules.running_updates += u16::try_from(tick - 20).unwrap();
            parts.rules.remaining_ticks -= u16::try_from(tick - 20).unwrap();
            let ObservedActor::Visible(cpu) = &mut parts.actors[SeatId::Two.index()] else {
                unreachable!()
            };
            cpu.position = position;
            cpu.heading = heading;
        })
        .unwrap()
}

fn heading(mask: u8) -> Direction {
    [
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ]
    .into_iter()
    .find(|direction| {
        mask & match direction {
            Direction::North => 1,
            Direction::East => 2,
            Direction::South => 4,
            Direction::West => 8,
        } != 0
    })
    .unwrap()
}
