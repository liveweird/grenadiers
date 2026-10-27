use grenadiers_core::cpu::survival::WORK_ALLOWANCE;
use grenadiers_core::game::{DrawReason, RoundResult};
use grenadiers_core::room::CpuProfile;
use serde_json::{Value, json};

use super::super::evaluation_support::{EvaluationCase, EvaluationRun, run};
use crate::game::rooms::measurement;

pub(super) fn emit_matrix(
    kind: &str,
    cases: Vec<EvaluationCase>,
    tick_cap: u64,
    profile: CpuProfile,
) {
    measurement::require_release();
    let source_identity = required_metadata("GRENADIERS_MEASUREMENT_SOURCE");
    let toolchain = required_metadata("GRENADIERS_MEASUREMENT_TOOLCHAIN");
    let mut planner_times = Vec::new();
    let mut results = Vec::new();
    for case in cases {
        let evaluation = run(case, profile).unwrap();
        assert!(evaluation.trace.replay_verified);
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
        kind,
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
                "tickCap": tick_cap,
                "hiddenRetries": 0,
            },
            "plannerTimings": measurement::summary_ns(&planner_times),
            "results": results,
            "interpretation": "fixed scripted workloads; not evidence of human-level strength",
        }),
    );
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
        "outcome": match trace.terminal {
            Some(RoundResult::Draw(DrawReason::Timeout)) => "natural-timeout-draw",
            Some(_) => "natural-result",
            None => "censored",
        },
        "censored": trace.censored,
        "replayVerified": trace.replay_verified,
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
