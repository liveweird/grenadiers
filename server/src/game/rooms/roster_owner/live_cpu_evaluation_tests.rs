use grenadiers_core::{cpu::survival::WORK_ALLOWANCE, room::CpuDifficulty};
use serde_json::json;

use super::{
    live_cpu_evaluation::{FULL_ROUND_CAP, run},
    live_cpu_evaluation_types::{EvaluationCase, ReferenceScript},
};
use crate::game::rooms::measurement;

const SIMULATION_ROOT_SEED: u8 = 113;

fn case(difficulty: CpuDifficulty) -> EvaluationCase {
    EvaluationCase {
        label: "live-difficulty-short",
        difficulty,
        arena_seed: 41,
        simulation_root_seed: SIMULATION_ROOT_SEED,
        cpu_seed_set: 71,
        script: ReferenceScript::FixedIdle,
    }
}

#[test]
fn live_difficulty_short_episodes_repeat_exactly_at_every_level() {
    for difficulty in [
        CpuDifficulty::Easy,
        CpuDifficulty::Standard,
        CpuDifficulty::Hard,
    ] {
        let first = run(case(difficulty), 180).unwrap();
        let second = run(case(difficulty), 180).unwrap();
        assert_eq!(
            first.measurement.deterministic,
            second.measurement.deterministic
        );
        assert_eq!(first.final_inspection, second.final_inspection);
        assert!(first.measurement.deterministic.shadow_exact);
        assert!(first.measurement.deterministic.replay_exact);
    }
}

#[test]
fn live_difficulty_seed_axes_are_independent() {
    let baseline = run(case(CpuDifficulty::Standard), 1).unwrap();
    let cpu_changed = run(
        EvaluationCase {
            cpu_seed_set: 91,
            ..case(CpuDifficulty::Standard)
        },
        1,
    )
    .unwrap();
    assert_eq!(baseline.initial_state, cpu_changed.initial_state);
    assert_ne!(baseline.cpu_seeds, cpu_changed.cpu_seeds);

    let arena_changed = run(
        EvaluationCase {
            arena_seed: 42,
            ..case(CpuDifficulty::Standard)
        },
        1,
    )
    .unwrap();
    assert_ne!(baseline.initial_state, arena_changed.initial_state);
    assert_eq!(baseline.cpu_seeds, arena_changed.cpu_seeds);
}

#[test]
fn live_difficulty_short_cap_is_reported_as_censored() {
    let evidence = run(
        EvaluationCase {
            label: "live-difficulty-short-censored",
            script: ReferenceScript::BombRetreat,
            ..case(CpuDifficulty::Hard)
        },
        2,
    )
    .unwrap();
    assert!(!evidence.measurement.deterministic.terminal);
    assert!(evidence.measurement.deterministic.censored);
    assert_eq!(evidence.measurement.deterministic.physical_round, 1);
}

#[test]
#[ignore = "full 48-case deterministic live round evaluation; run through --cpu-live-rounds"]
fn live_difficulty_round_evaluation_measurement() {
    measurement::require_release();
    let mut planner_times = Vec::new();
    let mut results = Vec::new();
    for arena_seed in [41, 53, 67, 79] {
        for cpu_seed_set in [71, 91] {
            for script in [ReferenceScript::FixedIdle, ReferenceScript::BombRetreat] {
                for difficulty in [
                    CpuDifficulty::Easy,
                    CpuDifficulty::Standard,
                    CpuDifficulty::Hard,
                ] {
                    let evidence = run(
                        EvaluationCase {
                            label: "live-difficulty-full-round",
                            difficulty,
                            arena_seed,
                            simulation_root_seed: SIMULATION_ROOT_SEED,
                            cpu_seed_set,
                            script,
                        },
                        FULL_ROUND_CAP,
                    )
                    .unwrap();
                    let result = &evidence.measurement.deterministic;
                    assert!(result.terminal, "{result:?}");
                    assert!(!result.censored, "{result:?}");
                    assert_eq!(result.physical_round, 1, "{result:?}");
                    assert!(result.shadow_exact && result.replay_exact, "{result:?}");
                    assert_eq!(result.queue.outstanding, 0, "{result:?}");
                    assert_eq!(result.queue.failed, 0, "{result:?}");
                    assert!(result.queue.max_work <= WORK_ALLOWANCE, "{result:?}");
                    if result.running_updates == FULL_ROUND_CAP as u64 {
                        assert_eq!(result.round_result.as_deref(), Some("Draw(Timeout)"));
                    }
                    planner_times.extend(evidence.measurement.planner_times);
                    results.push(serde_json::to_value(result).unwrap());
                }
            }
        }
    }
    assert_eq!(results.len(), 48);
    assert!(!planner_times.is_empty());

    let mut metadata = measurement::metadata();
    let metadata = metadata.as_object_mut().unwrap();
    metadata.insert("roomProfile".to_owned(), json!("LiveRosterV21"));
    metadata.insert(
        "recordingPolicy".to_owned(),
        json!("evaluation-test-recording-enabled"),
    );
    measurement::emit(
        "cpu-live-rounds",
        json!({
            "metadata": metadata,
            "identity": {
                "replayFormat": 35,
                "generatorVersion": 7,
                "spikePitLayoutVersion": 2,
                "gameplayVersion": 21,
                "punchActivation": "ContactV2",
                "easyPolicyVersion": 62,
                "standardPolicyVersion": 62,
                "hardPolicyVersion": 66,
                "enabledItems": "all",
            },
            "matrix": {
                "arenaSeeds": 4,
                "cpuSeedSets": 2,
                "scripts": ["fixed-idle", "bomb-retreat"],
                "difficultyLevels": ["Easy", "Standard", "Hard"],
                "simulationRootSeedAxis": "fixed-explicit",
                "cases": 48,
                "runningUpdateCap": FULL_ROUND_CAP,
                "hiddenRetries": 0,
            },
            "plannerTimings": measurement::summary_ns(&planner_times),
            "results": results,
            "interpretation": "fixed scripted workloads; not evidence of human-level strength",
        }),
    );
}
