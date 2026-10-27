use std::collections::BTreeSet;

use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use serde_json::{Value, json};

use super::evaluation_support::{EvaluationCase, EvaluationRun, HumanScript, default_cases, run};
use crate::game::rooms::measurement;

#[test]
fn cpu_evaluation_exact_repeat_is_bounded_and_censored() {
    let first = run(routine_case("repeat", [11; 32], [17; 32], 80)).unwrap();
    let second = run(routine_case("repeat", [11; 32], [17; 32], 80)).unwrap();
    let bounded = run(routine_case("bounded", [11; 32], [17; 32], 1)).unwrap();

    assert_eq!(first.trace, second.trace);
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
    let arena_a = run(routine_case("arena-a", [11; 32], [17; 32], 120)).unwrap();
    let arena_b = run(routine_case("arena-b", [37; 32], [17; 32], 120)).unwrap();
    let cpu_b = run(EvaluationCase::new(
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
fn bomb_retreat_script_uses_reserved_spawn_cells() {
    let evaluation = run(EvaluationCase::new(
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
#[ignore = "isolated release evaluation; run through scripts/measure-game.sh --cpu"]
fn cpu_scripted_evaluation_measurement() {
    measurement::require_release();
    let source_identity = required_metadata("GRENADIERS_MEASUREMENT_SOURCE");
    let toolchain = required_metadata("GRENADIERS_MEASUREMENT_TOOLCHAIN");
    let mut planner_times = Vec::new();
    let mut results = Vec::new();
    for case in default_cases() {
        let evaluation = run(case).unwrap();
        assert!(
            evaluation
                .trace
                .planning
                .work
                .iter()
                .all(|work| *work <= WORK_ALLOWANCE)
        );
        planner_times.extend(evaluation.planner_times.iter().copied());
        results.push(case_report(&evaluation));
    }
    assert_eq!(results.len(), 16);
    assert!(!planner_times.is_empty());

    measurement::emit(
        "solo-scripted-evaluation",
        json!({
            "sourceIdentity": source_identity,
            "toolchain": toolchain,
            "invocationDeclaredCargoProfile": "release",
            "debugAssertions": cfg!(debug_assertions),
            "roomProfile": "ExpandedV5",
            "recordingPolicy": "evaluation-test-recording-enabled",
            "matrix": {
                "arenaSeeds": 4,
                "cpuSeeds": 2,
                "scripts": ["idle", "bomb-retreat"],
                "cases": 16,
                "tickCap": 1_800,
                "hiddenRetries": 0,
            },
            "plannerTimings": measurement::summary_ns(&planner_times),
            "results": results,
            "interpretation": "fixed scripted workloads; not evidence of human-level strength",
        }),
    );
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

fn case_report(evaluation: &EvaluationRun) -> Value {
    let trace = &evaluation.trace;
    json!({
        "case": trace.case_id,
        "arenaSeed": trace.arena_seed_id,
        "cpuSeed": trace.cpu_seed_id,
        "script": trace.script,
        "tickCap": trace.tick_cap,
        "profile": format!("{:?}", trace.profile),
        "settings": format!("{:?}", trace.settings),
        "source": trace.source,
        "schedule": trace.schedule,
        "terminal": trace.terminal.map(|terminal| format!("{terminal:?}")),
        "censored": trace.censored,
        "runningTicks": trace.running_ticks,
        "cpuAlive": trace.cpu_alive,
        "acceptedMovementActions": trace.accepted_movement_actions,
        "acceptedManualActions": trace.accepted_manual_actions,
        "firstObservedCpuBombPlacedTick": trace
            .first_observed_cpu_bomb_placed_tick
            .map(|tick| tick.0),
        "firstObservedCpuPickupTick": trace.first_observed_cpu_pickup_tick.map(|tick| tick.0),
        "visitedCells": trace.visited_cells.iter().map(|cell| json!({
            "column": cell.column,
            "row": cell.row,
        })).collect::<Vec<_>>(),
        "stationaryTicks": trace.stationary_ticks,
        "planning": {
            "attempts": trace.planning.attempts,
            "delivered": trace.planning.delivered,
            "cancelled": trace.planning.cancelled,
            "failed": trace.planning.failed,
            "dropped": trace.planning.dropped,
            "work": work_summary(&trace.planning.work),
        },
        "workSum": trace.work_sum,
        "workMax": trace.work_max,
        "rngDraws": trace.rng_draws,
    })
}

fn work_summary(samples: &[u32]) -> Value {
    if samples.is_empty() {
        return json!({ "count": 0, "sum": 0, "max": 0 });
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    json!({
        "count": sorted.len(),
        "sum": sorted.iter().map(|value| u64::from(*value)).sum::<u64>(),
        "p50": percentile(&sorted, 50),
        "p95": percentile(&sorted, 95),
        "p99": percentile(&sorted, 99),
        "max": sorted.last().copied().unwrap(),
    })
}

fn percentile(sorted: &[u32], value: usize) -> u32 {
    let rank = (value * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn required_metadata(name: &str) -> String {
    let value = std::env::var(name).unwrap_or_else(|_| panic!("{name} must be explicit"));
    assert!(!value.trim().is_empty(), "{name} must not be blank");
    value
}
