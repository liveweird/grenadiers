use std::collections::BTreeSet;

use grenadiers_core::game::{CellId, Position};

use super::evaluation_support::{EvaluationCase, HumanScript, run};

const LATE_PROGRESS_BOUNDARY: u64 = 2_200;
const TICK_CAP: u64 = 3_600;

#[derive(Clone, Copy)]
struct TurnAlignmentCase {
    id: &'static str,
    arena_id: &'static str,
    arena_seed: [u8; 32],
}

#[test]
fn a0_c0_queued_cpu_crosses_the_confirmed_turn_stall_after_tick_2200() {
    assert_post_stall_progress(TurnAlignmentCase {
        id: "a0-c0-turn-alignment",
        arena_id: "a0",
        arena_seed: [11; 32],
    });
}

#[test]
fn a2_c0_queued_cpu_crosses_the_confirmed_turn_stall_after_tick_2200() {
    assert_post_stall_progress(TurnAlignmentCase {
        id: "a2-c0-turn-alignment",
        arena_id: "a2",
        arena_seed: [73; 32],
    });
}

fn assert_post_stall_progress(case: TurnAlignmentCase) {
    let evaluation_case = || {
        EvaluationCase::new(
            case.id,
            case.arena_id,
            case.arena_seed,
            "c0",
            [17; 32],
            HumanScript::BombRetreat,
            TICK_CAP,
        )
    };
    let first = run(evaluation_case()).unwrap().trace;
    let repeat = run(evaluation_case()).unwrap().trace;

    assert!(first.replay_verified && repeat.replay_verified);
    assert_eq!(
        first.accepted_actions, repeat.accepted_actions,
        "{} must replay the exact accepted input stream",
        case.id
    );
    assert_eq!(
        first, repeat,
        "{} must produce a deterministic physical trace",
        case.id
    );
    assert_eq!(first.running_ticks, TICK_CAP);
    assert!(first.censored);
    assert!(first.terminal.is_none());
    assert!(first.cpu_alive);
    assert!(first.frames.iter().all(|frame| frame.cpu_alive));

    let position_at_boundary = first
        .frames
        .iter()
        .find(|frame| frame.running_tick == LATE_PROGRESS_BOUNDARY)
        .map(|frame| frame.cpu_position)
        .expect("3600-tick evaluation must include the stall boundary");
    let cells_before_or_at_boundary = first
        .frames
        .iter()
        .take_while(|frame| frame.running_tick <= LATE_PROGRESS_BOUNDARY)
        .filter_map(|frame| frame.cpu_position.cell())
        .collect::<BTreeSet<_>>();
    let later_positions = first
        .frames
        .iter()
        .filter(|frame| frame.running_tick > LATE_PROGRESS_BOUNDARY)
        .map(|frame| frame.cpu_position)
        .collect::<Vec<_>>();

    assert_physical_movement(case.id, position_at_boundary, &later_positions);
    assert_new_cell_progress(case.id, &cells_before_or_at_boundary, &later_positions);
}

fn assert_physical_movement(case_id: &str, boundary: Position, later: &[Position]) {
    assert!(
        later.iter().any(|position| *position != boundary),
        "{case_id} must physically move after tick {LATE_PROGRESS_BOUNDARY}"
    );
}

fn assert_new_cell_progress(case_id: &str, earlier_cells: &BTreeSet<CellId>, later: &[Position]) {
    assert!(
        later
            .iter()
            .filter_map(|position| position.cell())
            .any(|cell| !earlier_cells.contains(&cell)),
        "{case_id} must reach a new cell after tick {LATE_PROGRESS_BOUNDARY}"
    );
}
