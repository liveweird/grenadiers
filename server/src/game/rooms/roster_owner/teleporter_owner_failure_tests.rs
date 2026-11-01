use grenadiers_core::{
    game::Direction,
    room::{CpuDifficulty, CpuProfile},
};
use tokio::time::Instant;

use super::teleporter_owner_support::{CPU, mailbox_is_cleared, test_driver};
use crate::game::rooms::roster_runtime::RosterAction;

#[test]
fn rejected_movement_never_commits_or_displaces_and_physically_fences() {
    let (mut owner, planner) = test_driver(
        CpuProfile::TELEPORTER_V46_STANDARD,
        CpuDifficulty::Standard,
        false,
        false,
    );
    let start = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .position();
    owner.continuation_rejections[0] = true;
    for _ in 0..30 {
        planner.pump_for_test(usize::MAX);
        owner.tick(Instant::now()).unwrap();
        if owner
            .teleporter_trace_for_test()
            .iter()
            .any(|(kind, _, _, accepted)| *kind == 0 && !accepted)
        {
            break;
        }
    }
    let trace = owner.teleporter_trace_for_test();
    assert!(
        trace
            .iter()
            .any(|(kind, _, _, accepted)| *kind == 0 && !accepted)
    );
    assert!(trace.iter().any(|(kind, _, _, _)| *kind == 2));
    assert!(!owner.cpus[0].teleporter_controlled());
    assert_eq!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .actor(CPU)
            .unwrap()
            .position(),
        start
    );
    assert!(mailbox_is_cleared(
        &owner.runtime.replay_inspection().inputs
    ));
}

#[test]
fn rejected_stop_still_clears_queued_movement_and_manual_action_after_one_move() {
    let (mut owner, planner) = test_driver(
        CpuProfile::TELEPORTER_V46_STANDARD,
        CpuDifficulty::Standard,
        false,
        false,
    );
    let start = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .position();
    let moved = run_until_first_move(&mut owner, &planner);
    assert_ne!(moved, start);

    let epoch = owner.runtime.lifecycle().round_attempt();
    owner
        .runtime
        .submit_cpu_intent(CPU, epoch, RosterAction::Movement(Some(Direction::East)))
        .unwrap();
    let manual = owner.runtime.next_cpu_manual_action(CPU).unwrap();
    owner
        .runtime
        .submit_cpu_intent(CPU, epoch, RosterAction::Bomb(manual))
        .unwrap();
    owner.continuation_rejections[1] = true;
    owner.tick(Instant::now()).unwrap();

    let trace = owner.teleporter_trace_for_test();
    assert!(
        trace
            .iter()
            .any(|(kind, _, _, accepted)| *kind == 1 && !accepted)
    );
    assert!(mailbox_is_cleared(
        &owner.runtime.replay_inspection().inputs
    ));
    assert!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .world()
            .bombs()
            .is_empty()
    );
    assert_eq!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .actor(CPU)
            .unwrap()
            .position(),
        moved
    );
    owner.tick(Instant::now()).unwrap();
    assert_eq!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .actor(CPU)
            .unwrap()
            .position(),
        moved
    );
    assert_eq!(accepted_movement_count(&owner), 1);
}

#[test]
fn missing_stopped_publications_expire_and_fence_before_fresh_stop_observation() {
    let (mut owner, planner) = test_driver(
        CpuProfile::TELEPORTER_V46_STANDARD,
        CpuDifficulty::Standard,
        false,
        false,
    );
    let moved = run_until_first_move(&mut owner, &planner);
    for _ in 0..30 {
        owner.service_cpu().unwrap();
        let _ = owner.runtime.settle_boundary([None; 4]).unwrap();
    }
    let trace = owner.teleporter_trace_for_test();
    assert!(trace.iter().any(|(kind, _, _, _)| *kind == 1));
    assert!(trace.iter().any(|(kind, _, _, _)| *kind == 2));
    assert_eq!(accepted_movement_count(&owner), 1);
    assert!(!owner.cpus[0].teleporter_controlled());
    assert_eq!(
        owner
            .runtime
            .state_for_test()
            .unwrap()
            .actor(CPU)
            .unwrap()
            .position(),
        moved
    );
    assert!(mailbox_is_cleared(
        &owner.runtime.replay_inspection().inputs
    ));

    let fresh = owner.runtime.current_notice().unwrap();
    owner.publish(fresh);
    owner.tick(Instant::now()).unwrap();
    assert_eq!(accepted_movement_count(&owner), 1);
}

fn run_until_first_move(
    owner: &mut super::driver::Driver,
    planner: &crate::game::rooms::cpu::CpuPlanner,
) -> grenadiers_core::game::Position {
    for _ in 0..30 {
        planner.pump_for_test(usize::MAX);
        owner.tick(Instant::now()).unwrap();
        if accepted_movement_count(owner) == 1 {
            return owner
                .runtime
                .state_for_test()
                .unwrap()
                .actor(CPU)
                .unwrap()
                .position();
        }
    }
    panic!("expected one admitted Teleporter movement")
}

fn accepted_movement_count(owner: &super::driver::Driver) -> usize {
    owner
        .teleporter_trace_for_test()
        .iter()
        .filter(|(kind, _, _, accepted)| *kind == 0 && *accepted)
        .count()
}
