use std::collections::BTreeSet;

use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use grenadiers_core::room::CpuProfile;

#[path = "cpu_evaluation_report.rs"]
mod report;

use super::evaluation_support::{
    EvaluationCase, HumanScript, default_cases, full_round_cases, run,
};

#[test]
fn cpu_evaluation_exact_repeat_is_bounded_and_censored() {
    let first = run_standard(routine_case("repeat", [11; 32], [17; 32], 80)).unwrap();
    let second = run_standard(routine_case("repeat", [11; 32], [17; 32], 80)).unwrap();
    let bounded = run_standard(routine_case("bounded", [11; 32], [17; 32], 1)).unwrap();

    assert_eq!(first.trace, second.trace);
    assert!(first.trace.replay_verified);
    assert!(!first.trace.accepted_actions.is_empty());
    assert_eq!(first.trace.accepted_actions, second.trace.accepted_actions);
    assert_eq!(first.trace.running_ticks, 80);
    assert_eq!(first.trace.frames.len(), 80);
    assert!(first.trace.terminal.is_none());
    assert!(first.trace.censored);
    assert!(first.trace.planning.attempts > 0);
    assert!(first.trace.planning.delivered > 0);
    assert!(first.trace.rng_draws <= first.trace.planning.delivered as u64);
    assert_eq!(first.trace.work_sum, second.trace.work_sum);
    assert!(
        first
            .trace
            .planning
            .work
            .iter()
            .all(|work| *work <= WORK_ALLOWANCE)
    );
    assert_eq!(
        first.trace.frames.last().map(|frame| frame.rng_draws),
        Some(first.trace.rng_draws)
    );
    assert_eq!(bounded.trace.running_ticks, 1);
    assert!(bounded.trace.censored);
    assert!(bounded.trace.terminal.is_none());
    assert!(bounded.trace.first_observed_cpu_bomb_placed_tick.is_none());
    assert!(bounded.trace.first_observed_cpu_pickup_tick.is_none());
}

#[test]
fn cpu_evaluation_separates_seed_axes_and_makes_generated_progress() {
    let arena_a = run_standard(routine_case("arena-a", [11; 32], [17; 32], 120)).unwrap();
    let arena_b = run_standard(routine_case("arena-b", [37; 32], [17; 32], 120)).unwrap();
    let cpu_b = run_standard(EvaluationCase::new(
        "cpu-b",
        "arena-a",
        [11; 32],
        "cpu-b",
        [151; 32],
        HumanScript::Idle,
        120,
    ))
    .unwrap();

    assert_eq!(arena_a.trace.cpu_seed_id, arena_b.trace.cpu_seed_id);
    assert_ne!(arena_a.trace.arena_seed_id, arena_b.trace.arena_seed_id);
    assert_ne!(
        arena_a.trace.initial_public_world,
        arena_b.trace.initial_public_world
    );
    assert_ne!(arena_a.trace.frames, arena_b.trace.frames);
    assert_eq!(arena_a.trace.arena_seed_id, cpu_b.trace.arena_seed_id);
    assert_ne!(arena_a.trace.cpu_seed_id, cpu_b.trace.cpu_seed_id);
    assert_eq!(
        arena_a.trace.initial_public_world,
        cpu_b.trace.initial_public_world
    );
    for evaluation in [&arena_a, &arena_b, &cpu_b] {
        assert!(evaluation.trace.accepted_movement_actions > 0);
        assert!(evaluation.trace.visited_cells.len() > 1);
        assert!(evaluation.trace.planning.attempts > 0);
        assert!(
            evaluation
                .trace
                .planning
                .work
                .iter()
                .all(|work| *work <= WORK_ALLOWANCE)
        );
    }
}

#[test]
fn easy_evaluation_is_separately_labeled_and_exactly_repeatable() {
    let case = routine_case("easy-repeat", [11; 32], [17; 32], 120);
    let first = run(case, CpuProfile::CURRENT_EASY).unwrap();
    let second = run(case, CpuProfile::CURRENT_EASY).unwrap();

    assert_eq!(first.trace.profile, CpuProfile::CURRENT_EASY);
    assert_eq!(first.trace, second.trace);
    assert!(first.trace.replay_verified);
    assert!(first.trace.accepted_movement_actions > 0);
}

#[test]
fn standard_opening_is_identical_to_the_archived_v15_policy() {
    let case = routine_case("standard-v15", [11; 32], [17; 32], 120);
    let mut current = run(case, CpuProfile::CURRENT).unwrap().trace;
    let archived = run(case, CpuProfile::EQUIPMENT_GOALS_V15).unwrap().trace;

    current.profile = CpuProfile::EQUIPMENT_GOALS_V15;
    assert_eq!(current, archived);
}

#[test]
#[ignore = "isolated release acceptance for the complete Standard parity matrices"]
fn cpu_standard_v15_matrix_parity_acceptance() {
    for case in default_cases().into_iter().chain(full_round_cases()) {
        let mut current = run(case, CpuProfile::CURRENT).unwrap().trace;
        let archived = run(case, CpuProfile::EQUIPMENT_GOALS_V15).unwrap().trace;
        current.profile = CpuProfile::EQUIPMENT_GOALS_V15;
        assert_eq!(
            current, archived,
            "case={} tick_cap={}",
            case.id, case.tick_cap
        );
    }
}

#[test]
fn easy_keeps_the_initial_decision_immediate() {
    let case = routine_case("initial-immediate", [11; 32], [17; 32], 80);
    let standard = run(case, CpuProfile::CURRENT).unwrap();
    let easy = run(case, CpuProfile::CURRENT_EASY).unwrap();
    let first_cpu_action = |evaluation: &super::evaluation_support::EvaluationRun| {
        evaluation
            .trace
            .accepted_actions
            .iter()
            .find(|(_, seat, _)| *seat == grenadiers_core::game::SeatId::Two)
            .copied()
    };

    assert_eq!(first_cpu_action(&easy), first_cpu_action(&standard));
    assert!(first_cpu_action(&easy).is_some());
}

#[test]
fn bomb_retreat_script_uses_reserved_spawn_cells() {
    let evaluation = run_standard(EvaluationCase::new(
        "bomb-retreat-reserved-opening",
        "arena-a",
        [11; 32],
        "cpu-a",
        [17; 32],
        HumanScript::BombRetreat,
        260,
    ))
    .unwrap();
    let placement = evaluation
        .trace
        .first_observed_human_bomb_placement
        .expect("the opening Bomb must be physically observed");
    assert_eq!(placement.tick.0, 202);
    assert_eq!(placement.cell, grenadiers_core::game::CellId::new(2, 1));
    for (running_tick, physical_tick, cell) in [
        (21, 201, grenadiers_core::game::CellId::new(2, 1)),
        (43, 223, grenadiers_core::game::CellId::new(1, 1)),
        (64, 244, grenadiers_core::game::CellId::new(1, 2)),
    ] {
        let frame = evaluation
            .trace
            .frames
            .iter()
            .find(|frame| frame.running_tick == running_tick)
            .unwrap();
        assert_eq!(frame.physical_tick.0, physical_tick);
        assert_eq!(
            frame.human_position,
            grenadiers_core::game::Position::center(cell)
        );
        assert!(frame.human_alive);
    }
    let detonation_tick = placement.tick.0 + evaluation.trace.settings.bomb_fuse_ticks();
    let detonation = evaluation
        .trace
        .frames
        .iter()
        .find(|frame| frame.physical_tick.0 >= detonation_tick)
        .expect("live run must observe the human bomb's detonation tick");
    assert!(detonation.human_alive);
    assert!(
        evaluation
            .trace
            .frames
            .iter()
            .all(|frame| frame.human_alive)
    );
}

#[test]
fn cpu_evaluation_default_matrix_is_complete_without_hidden_retries() {
    let cases = default_cases();
    assert_eq!(cases.len(), 16);
    assert!(cases.iter().all(|case| case.tick_cap == 1_800));
    assert_eq!(
        cases
            .iter()
            .map(|case| case.id)
            .collect::<BTreeSet<_>>()
            .len(),
        16
    );
    let combinations = cases
        .iter()
        .map(|case| (case.arena_seed_id, case.cpu_seed_id, case.script.name()))
        .collect::<BTreeSet<_>>();
    assert_eq!(combinations.len(), 16);
    assert_eq!(
        cases
            .iter()
            .filter(|case| case.script == HumanScript::Idle)
            .count(),
        8
    );
    assert_eq!(
        cases
            .iter()
            .filter(|case| case.script == HumanScript::BombRetreat)
            .count(),
        8
    );
}

#[test]
fn full_round_matrix_changes_only_the_declared_tick_cap() {
    let short = default_cases();
    let full = full_round_cases();
    assert_eq!(full.len(), 16);
    for (short, full) in short.into_iter().zip(full) {
        assert_eq!(short.id, full.id);
        assert_eq!(short.arena_seed_id, full.arena_seed_id);
        assert_eq!(short.cpu_seed_id, full.cpu_seed_id);
        assert_eq!(short.script, full.script);
        assert_eq!(short.tick_cap, 1_800);
        assert_eq!(full.tick_cap, 10_800);
    }
}

#[test]
#[ignore = "isolated release evaluation; run through scripts/measure-game.sh --cpu"]
fn cpu_scripted_evaluation_measurement() {
    report::emit_matrix(
        "solo-scripted-evaluation",
        default_cases(),
        1_800,
        CpuProfile::CURRENT,
    );
}

#[test]
#[ignore = "isolated full-round evaluation; run through scripts/measure-game.sh --cpu-rounds"]
fn cpu_full_round_evaluation_measurement() {
    report::emit_matrix(
        "solo-full-round-evaluation",
        full_round_cases(),
        10_800,
        CpuProfile::CURRENT,
    );
}

#[test]
#[ignore = "isolated Easy release evaluation; run through scripts/measure-game.sh --cpu-easy"]
fn cpu_easy_scripted_evaluation_measurement() {
    report::emit_matrix(
        "solo-easy-scripted-evaluation",
        default_cases(),
        1_800,
        CpuProfile::CURRENT_EASY,
    );
}

#[test]
#[ignore = "isolated Easy full-round evaluation; run through scripts/measure-game.sh --cpu-easy-rounds"]
fn cpu_easy_full_round_evaluation_measurement() {
    report::emit_matrix(
        "solo-easy-full-round-evaluation",
        full_round_cases(),
        10_800,
        CpuProfile::CURRENT_EASY,
    );
}

fn run_standard(
    case: EvaluationCase,
) -> Result<
    super::evaluation_support::EvaluationRun,
    crate::game::rooms::match_runtime::MatchRuntimeError,
> {
    run(case, CpuProfile::CURRENT)
}

fn routine_case(
    id: &'static str,
    arena_seed: [u8; 32],
    cpu_seed: [u8; 32],
    tick_cap: u64,
) -> EvaluationCase {
    EvaluationCase::new(
        id,
        id,
        arena_seed,
        "cpu-a",
        cpu_seed,
        HumanScript::Idle,
        tick_cap,
    )
}
