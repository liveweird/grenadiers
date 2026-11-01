use crate::cpu::policy::UtilityMemory;
use crate::cpu::{Observation, ObservedActor, WorkBudget};
use crate::game::{Direction, Tick};

use super::teleporter_execution_tests::{
    after_move, at, basis, decision, key, plan, step, step_direction,
};
use super::{
    PreparedTeleporterStep, TELEPORTER_ADMISSION_ALLOWANCE, TeleporterStepPlan,
    TeleporterStepStatus,
};

#[test]
fn perpendicular_alignment_only_update_retains_old_heading_and_completes() {
    let basis = basis()
        .map_parts(|parts| {
            let ObservedActor::Visible(root) = &mut parts.actors[2] else {
                unreachable!()
            };
            root.position.y += 128;
            root.heading = Direction::North;
        })
        .unwrap();
    let certified = step_direction(&basis, Tick(6_111), Direction::West).unwrap();
    let start = recipient(&basis).position;
    let expected = crate::game::Position {
        x: start.x,
        // Base speed advances exactly 51 units on this first zero-residual update.
        y: start.y - 51,
    };
    assert!(certified.allows_stopped_position(expected));
    let memory = UtilityMemory::new(key(&basis));
    let mut prepared = PreparedTeleporterStep::admit(
        TeleporterStepPlan::new_for_test(certified, memory),
        basis.clone(),
        &basis,
        Tick(6_111),
        decision(&basis),
        basis.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    )
    .unwrap();
    let movement = prepared.command().unwrap();
    let mut committed = prepared.commit(movement).unwrap();
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), memory.key(), 7)
    else {
        unreachable!()
    };
    assert!(committed.record_stop(stop));
    let stopped = at(&basis, 2)
        .map_parts(|parts| {
            let ObservedActor::Visible(root) = &mut parts.actors[2] else {
                unreachable!()
            };
            root.position = expected;
            root.heading = Direction::North;
        })
        .unwrap();
    assert!(matches!(
        committed.observe_stopped(
            &stopped,
            Tick(6_123),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        ),
        TeleporterStepStatus::Completed(_)
    ));
}

#[test]
fn stopped_observation_has_exact_and_one_short_shared_budget() {
    let basis = basis();
    let stopped = after_move(&basis, 2, Direction::West);
    let mut measured = WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE);
    assert!(matches!(
        committed(&basis).observe_stopped(&stopped, Tick(6_123), &mut measured),
        TeleporterStepStatus::Completed(_)
    ));
    let required = measured.spent();
    assert!((1..=TELEPORTER_ADMISSION_ALLOWANCE).contains(&required));
    for (limit, completed) in [(required, true), (required - 1, false)] {
        let status =
            committed(&basis).observe_stopped(&stopped, Tick(6_123), &mut WorkBudget::new(limit));
        assert_eq!(
            matches!(status, TeleporterStepStatus::Completed(_)),
            completed
        );
    }
}

fn committed(basis: &Observation) -> super::CommittedTeleporterStep {
    let memory = UtilityMemory::new(key(basis));
    let mut prepared = PreparedTeleporterStep::admit(
        plan(step(basis, Tick(6_111)).unwrap()),
        basis.clone(),
        basis,
        Tick(6_111),
        decision(basis),
        basis.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    )
    .unwrap();
    let movement = prepared.command().unwrap();
    let mut committed = prepared.commit(movement).unwrap();
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), memory.key(), 7)
    else {
        unreachable!()
    };
    assert!(committed.record_stop(stop));
    committed
}

fn recipient(observation: &Observation) -> &crate::cpu::VisibleActor {
    observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
        .unwrap()
}
