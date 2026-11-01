use crate::cpu::{Observation, ObservedActor, WorkBudget};
use crate::game::{Direction, Equipment, SpecialWeapon, Tick};

use super::teleporter_execution_tests::{after_move, at, basis, decision, key, plan, step};
use super::{PreparedTeleporterStep, TELEPORTER_ADMISSION_ALLOWANCE, TeleporterStepStatus};

fn dead_roster_basis() -> Observation {
    basis()
        .map_parts(|parts| {
            let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                unreachable!()
            };
            rival.alive = false;
            rival.equipment = None;
            rival.weapon = None;
            rival.effects.clear();
            rival.damage_grace_remaining_ticks = 0;
        })
        .unwrap()
}

fn admit(
    basis: &Observation,
    current: &Observation,
) -> Result<PreparedTeleporterStep, crate::cpu::routing::RouteFailure> {
    PreparedTeleporterStep::admit_v48(
        plan(step(basis, Tick(6_111)).unwrap()),
        basis.clone(),
        current,
        Tick(6_111),
        decision(basis),
        current.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    )
}

#[test]
fn unchanged_dead_rival_allows_admission_and_mandatory_stopped_completion() {
    let basis = dead_roster_basis();
    // Certification already accepts a visible dead roster member. Admission
    // must preserve that fact instead of requiring every member to be alive.
    let mut prepared = admit(&basis, &at(&basis, 1)).unwrap();
    let movement = prepared.command().unwrap();
    let mut committed = prepared.commit(movement).unwrap();
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_123), key(&basis), 7)
    else {
        panic!("movement must be followed by the sealed Stop")
    };
    assert!(committed.record_stop(stop));
    let status = committed.observe_stopped(
        &after_move(&basis, 3, Direction::West),
        Tick(6_124),
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    );
    assert!(matches!(status, TeleporterStepStatus::Completed(_)));
    assert!(committed.next_memory().is_some());
}

#[test]
fn archived_constructor_preserves_dead_roster_refusal() {
    let basis = dead_roster_basis();
    let current = at(&basis, 1);
    let result = PreparedTeleporterStep::admit(
        plan(step(&basis, Tick(6_111)).unwrap()),
        basis.clone(),
        &current,
        Tick(6_111),
        decision(&basis),
        current.parts().tick,
        &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
    );
    assert!(result.is_err());
}

#[test]
fn dead_roster_resurrection_displacement_or_capability_change_rejects() {
    let basis = dead_roster_basis();
    for change in 0..8 {
        let changed = at(&basis, 1)
            .map_parts(|parts| {
                let ObservedActor::Visible(rival) = &mut parts.actors[0] else {
                    unreachable!()
                };
                match change {
                    0 => rival.alive = true,
                    1 => rival.position.x += 1,
                    2 => {
                        rival.heading = match rival.heading {
                            Direction::North => Direction::South,
                            _ => Direction::North,
                        }
                    }
                    3 => rival.bomb_capacity += 1,
                    4 => rival.weapon = Some(SpecialWeapon::Power),
                    5 => rival.last_teleport_tick = Some(parts.tick),
                    6 => rival.live_bomb_count += 1,
                    7 => rival.equipment = Some(Equipment::Punch),
                    _ => unreachable!(),
                }
            })
            .unwrap();
        assert!(admit(&basis, &changed).is_err(), "case {change}");
    }
}

#[test]
fn current_executor_commits_memory_after_late_physical_stop_confirmation() {
    let basis = basis();
    let stopped = after_move(&basis, 3, Direction::West);
    for (current_policy, completes) in [(true, true), (false, false)] {
        let mut committed = committed(&basis, current_policy, true);
        let status = committed.observe_stopped(
            &stopped,
            Tick(6_124),
            &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
        );
        assert_eq!(
            matches!(status, TeleporterStepStatus::Completed(_)),
            completes
        );
        assert_eq!(committed.next_memory().is_some(), completes);
    }
}

#[test]
fn current_executor_rejects_unproven_stop_or_expired_or_changed_scene() {
    let basis = basis();
    let stopped = after_move(&basis, 3, Direction::West);
    // Actual physical Stop remains 6,123. A later publication cannot move that
    // deadline, forge confirmation before it, or claim future knowledge.
    for (confirmation, stop_recorded) in [
        (Tick(6_122), true),
        (Tick(6_125), true),
        (Tick(6_124), false),
    ] {
        assert!(matches!(
            committed(&basis, true, stop_recorded).observe_stopped(
                &stopped,
                confirmation,
                &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
            ),
            TeleporterStepStatus::Invalidated
        ));
    }
    for changed in [
        after_move(&basis, 18, Direction::West),
        stopped
            .map_parts(|parts| {
                let ObservedActor::Visible(actor) = &mut parts.actors[2] else {
                    unreachable!()
                };
                actor.position.x += 1_000;
            })
            .unwrap(),
    ] {
        assert!(matches!(
            committed(&basis, true, true).observe_stopped(
                &changed,
                changed.parts().tick,
                &mut WorkBudget::new(TELEPORTER_ADMISSION_ALLOWANCE),
            ),
            TeleporterStepStatus::Invalidated
        ));
    }
}

fn committed(
    basis: &Observation,
    current_policy: bool,
    record_stop: bool,
) -> super::CommittedTeleporterStep {
    let constructor = if current_policy {
        PreparedTeleporterStep::admit_v48
    } else {
        PreparedTeleporterStep::admit
    };
    let mut prepared = constructor(
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
    let TeleporterStepStatus::Submit(stop) = committed.poll_command(Tick(6_122), key(basis), 7)
    else {
        panic!("movement must be followed by the sealed Stop")
    };
    if record_stop {
        assert!(committed.record_stop(stop));
    }
    committed
}
