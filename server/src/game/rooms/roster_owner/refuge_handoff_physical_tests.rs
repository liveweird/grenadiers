use grenadiers_core::{
    game::{Direction, SeatId},
    room::CpuProfile,
};

use super::refuge_handoff_physical_support::{CPU, run, running_runtime_and_scene};
use crate::game::rooms::roster_runtime::{RosterAction, RosterRuntimeError};

#[test]
fn running_scene_installation_is_fresh_reset_and_transactional() {
    let (mut runtime, scene) = running_runtime_and_scene();
    assert!(runtime.pending_report_count_for_test() > 0);
    let epoch = runtime.lifecycle().round_attempt();
    runtime
        .action(
            SeatId::One,
            epoch,
            RosterAction::Movement(Some(Direction::West)),
        )
        .unwrap();
    let expected_tick = scene.tick();
    let expected_position = scene.actor(SeatId::One).unwrap().position();

    let mut rejected = scene.clone();
    rejected.step(&[]).unwrap();
    assert_eq!(
        runtime.install_running_scene_for_test(rejected),
        Err(RosterRuntimeError::Publication)
    );
    assert_eq!(runtime.state_for_test().unwrap().tick(), expected_tick);
    assert!(runtime.pending_report_count_for_test() > 0);

    let notice = runtime.install_running_scene_for_test(scene).unwrap();
    assert!(notice.spike_v19_cpu_publication().is_some());
    assert_eq!(
        runtime.publication_caches_for_test(),
        [false, false, false, false, false, true]
    );
    assert_eq!(runtime.pending_report_count_for_test(), 0);
    runtime.settle_boundary([None; 4]).unwrap();
    assert_eq!(
        runtime
            .state_for_test()
            .unwrap()
            .actor(SeatId::One)
            .unwrap()
            .position(),
        expected_position,
        "the queued pre-install movement was reset"
    );
}

#[test]
fn hard35_owner_places_two_owned_bombs_before_the_first_detonation_and_survives() {
    let outcome = run(CpuProfile::HARD_V35);
    let second = outcome.second_birth.expect("Hard35 replacement planted");

    assert!(second > outcome.first_birth);
    assert!(second < outcome.first_detonation);
    assert!(outcome.simultaneous_owned >= 2);
    assert!(outcome.selected_rival);
    assert!(outcome.rival_in_second_blast);
    assert_eq!(
        outcome.second_cell,
        Some(grenadiers_core::game::CellId::new(6, 3))
    );
    assert_eq!(outcome.cpu_at_second, outcome.second_cell);
    assert_eq!(
        outcome.rival_at_second,
        Some(grenadiers_core::game::CellId::new(9, 3))
    );
    assert!(outcome.completed_blast_lifetimes);
    assert!(outcome.cpu_alive_after_blasts);
    assert!(outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
}

#[test]
fn archived_hard34_keeps_the_original_hold_without_an_early_second_birth() {
    let outcome = run(CpuProfile::HARD_V34);

    assert!(outcome.retained_route_before_first_detonation);
    assert!(outcome.completed_blast_lifetimes);
    assert!(
        outcome
            .second_birth
            .is_none_or(|birth| birth >= outcome.first_detonation)
    );
    assert!(outcome.simultaneous_owned <= 1);
    assert!(outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    assert_eq!(CPU, SeatId::Two);
}
