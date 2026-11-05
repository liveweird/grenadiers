use std::collections::BTreeMap;

use grenadiers_core::room::{CpuDifficulty, CpuProfile};
use serde::Serialize;

pub(super) const CPU_SEATS: [grenadiers_core::game::SeatId; 3] = [
    grenadiers_core::game::SeatId::Two,
    grenadiers_core::game::SeatId::Three,
    grenadiers_core::game::SeatId::Four,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum ReferenceScript {
    FixedIdle,
    BombRetreat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EvaluationCase {
    pub label: &'static str,
    pub difficulty: CpuDifficulty,
    pub arena_seed: u8,
    pub simulation_root_seed: u8,
    pub cpu_seed_set: u8,
    pub script: ReferenceScript,
}

impl EvaluationCase {
    pub(super) const fn profile(self) -> CpuProfile {
        match self.difficulty {
            CpuDifficulty::Easy => CpuProfile::RECOVERY_V62_EASY,
            CpuDifficulty::Standard => CpuProfile::RECOVERY_V62_STANDARD,
            CpuDifficulty::Hard => CpuProfile::HARD_V66,
        }
    }

    pub(super) fn cpu_seeds(self) -> [[u8; 32]; 3] {
        std::array::from_fn(|index| {
            [self.cpu_seed_set.wrapping_add(u8::try_from(index).unwrap()); 32]
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(super) struct EventMetrics {
    pub bomb_placed: [u64; 4],
    pub bomb_detonated: [u64; 4],
    pub pickup_collected: [u64; 4],
    pub actor_eliminated: [u64; 4],
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(super) struct MobilityMetrics {
    pub distinct_cells: usize,
    pub cell_transitions: u64,
    pub stationary_updates: u64,
    pub longest_stationary_span: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(super) struct IntentMetrics {
    pub phases: BTreeMap<String, u64>,
    pub targets: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(super) struct AdmissionMetrics {
    pub expired_or_wrong_decision: u32,
    pub missed_manual_tick: u32,
    pub stale_scene: u32,
    pub invalid_hold: u32,
    pub contact: u32,
    pub strategic_plant: u32,
    pub invalid_shape: u32,
    pub pulse: u32,
    pub route: u32,
    pub duplicate_decision: u32,
    pub refuge_handoff: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub(super) struct QueueMetrics {
    pub dequeued: usize,
    pub delivered: usize,
    pub cancelled: usize,
    pub failed: usize,
    pub dropped: usize,
    pub max_work: u32,
    pub outstanding: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct DeterministicMeasurement {
    pub label: &'static str,
    pub difficulty: String,
    pub profile_version: u16,
    pub arena_seed: u8,
    pub simulation_root_seed: u8,
    pub cpu_seed_set: u8,
    pub cpu_seed_fingerprints: [u8; 3],
    pub script: ReferenceScript,
    pub running_updates: u64,
    pub terminal: bool,
    pub censored: bool,
    pub round_result: Option<String>,
    pub physical_round: u64,
    pub events: EventMetrics,
    pub mobility: [MobilityMetrics; 4],
    pub block_delta_unattributed: usize,
    pub intents: [IntentMetrics; 3],
    pub admissions: [AdmissionMetrics; 3],
    pub queue: QueueMetrics,
    pub rng_draws: [u64; 3],
    pub shadow_exact: bool,
    pub replay_exact: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(super) struct EpisodeMeasurement {
    pub deterministic: DeterministicMeasurement,
    #[serde(skip)]
    pub planner_times: Vec<std::time::Duration>,
}

pub(super) struct EpisodeEvidence {
    pub measurement: EpisodeMeasurement,
    pub initial_state: grenadiers_core::game::State,
    pub final_inspection: crate::game::rooms::roster_runtime::RosterRuntimeInspection,
    pub cpu_seeds: [[u8; 32]; 3],
}
