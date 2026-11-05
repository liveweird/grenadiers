use grenadiers_core::{cpu::survival::WORK_ALLOWANCE, room::CpuDifficulty};
use serde_json::json;

use super::{
    live_cpu_evaluation::{FULL_ROUND_CAP, run},
    live_cpu_evaluation_types::{EvaluationCase, ReferenceScript},
};
use crate::game::rooms::measurement;

#[test]
#[ignore = "192-case live seed sweep; run scripts/measure-game.sh --cpu-live-soak"]
fn live_difficulty_seed_sweep_soak() {
    measurement::require_release();
    let arena_seeds = [3, 7, 11, 17, 23, 29, 31, 37];
    let simulation_roots = [113, 197];
    let cpu_seeds = [71, 91];
    let mut results = Vec::new();
    let mut planner_times = Vec::new();
    for arena_seed in arena_seeds {
        for simulation_root_seed in simulation_roots {
            for cpu_seed_set in cpu_seeds {
                for script in [ReferenceScript::FixedIdle, ReferenceScript::BombRetreat] {
                    for difficulty in [
                        CpuDifficulty::Easy,
                        CpuDifficulty::Standard,
                        CpuDifficulty::Hard,
                    ] {
                        let case = EvaluationCase {
                            label: "live-seed-soak",
                            difficulty,
                            arena_seed,
                            simulation_root_seed,
                            cpu_seed_set,
                            script,
                        };
                        eprintln!("LIVE_CPU_SOAK_CASE_START {case:?}");
                        let evidence = run(case, FULL_ROUND_CAP)
                            .unwrap_or_else(|error| panic!("{case:?}: {error}"));
                        let result = &evidence.measurement.deterministic;
                        assert!(result.terminal && !result.censored, "{result:?}");
                        assert_eq!(result.physical_round, 1, "{result:?}");
                        assert!(result.shadow_exact && result.replay_exact, "{result:?}");
                        assert_eq!(result.queue.outstanding, 0, "{result:?}");
                        assert_eq!(result.queue.failed, 0, "{result:?}");
                        assert_eq!(result.queue.dropped, 0, "{result:?}");
                        assert!(result.queue.delivered > 0, "{result:?}");
                        assert!(result.queue.max_work <= WORK_ALLOWANCE, "{result:?}");
                        if result.mobility[1..]
                            .iter()
                            .any(|mobility| mobility.longest_stationary_span >= 600)
                        {
                            eprintln!(
                                "LIVE_CPU_SOAK_LONG_STATIONARY {case:?} final={:?}",
                                evidence.final_inspection
                            );
                        }
                        planner_times.extend(evidence.measurement.planner_times);
                        results.push(serde_json::to_value(result).unwrap());
                    }
                }
            }
        }
    }
    assert_eq!(results.len(), 192);
    let mut metadata = measurement::metadata();
    metadata["roomProfile"] = json!("LiveRosterV21");
    metadata["recordingPolicy"] = json!("evaluation-test-recording-enabled");
    measurement::emit(
        "cpu-live-soak",
        json!({
            "metadata": metadata,
            "matrix": {
                "arenaSeeds": arena_seeds,
                "simulationRootSeeds": simulation_roots,
                "cpuSeedSets": cpu_seeds,
                "scripts": ["fixed-idle", "bomb-retreat"],
                "difficultyLevels": ["Easy", "Standard", "Hard"],
                "cases": 192,
                "runningUpdateCap": FULL_ROUND_CAP,
                "hiddenRetries": 0,
            },
            "plannerTimings": measurement::summary_ns(&planner_times),
            "results": results,
            "interpretation": "seed sweep; stationary spans flag investigation, not proven stalls or human difficulty",
        }),
    );
}
