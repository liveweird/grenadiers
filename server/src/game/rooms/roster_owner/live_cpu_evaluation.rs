use std::sync::Arc;

#[path = "live_cpu_evaluation_identity.rs"]
mod identity;

use grenadiers_core::{
    game::{Direction, EnabledItems, MobNavigationPolicy, SeatId, StormCloudConfig},
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuDifficulty, LairRoomConfiguration,
        RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings,
        SpikePitTuning, SpikeRoomConfiguration,
    },
};
use tokio::{sync::watch, time::Instant};

pub(super) use super::live_cpu_evaluation_recording::{drain_trace, start_round};
use super::{
    super::{
        RandomFill,
        cpu::{CpuController, CpuPlanner, PumpReport},
        replay_v39, replay_v40,
        roster_runtime::{RosterAction, RosterRuntime},
    },
    RosterOwnerNotice,
    driver::Driver,
    live_cpu_evaluation_metrics::{MetricsCollector, admissions, cadence_metrics, rng_draws},
    live_cpu_evaluation_recording::EvaluationRecording,
    live_cpu_evaluation_types::{
        CPU_SEATS, DeterministicMeasurement, EpisodeEvidence, EpisodeMeasurement, EvaluationCase,
        QueueMetrics, ReferenceScript,
    },
};
use crate::round_seed::RoundSeedSource;
use identity::assert_identity;

pub(super) const FULL_ROUND_CAP: usize = 10_800;

pub(super) fn run(case: EvaluationCase, tick_cap: usize) -> Result<EpisodeEvidence, String> {
    let shadow = replay_v40::RecordingRosterRuntime::new_for_evaluation(
        configuration(case.difficulty),
        settings(),
        generation_seeds(case.arena_seed),
        [case.simulation_root_seed; 32],
        cpu_provenance(case),
    )
    .map_err(|error| format!("construct Replay40 evaluation fixture: {error:?}"))?;
    run_with_recording(case, tick_cap, shadow, MobNavigationPolicy::ReachedTargetV4)
}

pub(super) fn run_archived_v39(
    case: EvaluationCase,
    tick_cap: usize,
) -> Result<EpisodeEvidence, String> {
    let shadow = replay_v39::RecordingRosterRuntime::new_for_evaluation(
        configuration(case.difficulty),
        settings(),
        generation_seeds(case.arena_seed),
        [case.simulation_root_seed; 32],
        cpu_provenance(case),
    )
    .map_err(|error| format!("construct Replay39 evaluation fixture: {error:?}"))?;
    run_with_recording(case, tick_cap, shadow, MobNavigationPolicy::TimedEscapeV3)
}

fn settings() -> RoomSettings {
    RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT)
}

fn generation_seeds(arena_seed: u8) -> RoundSeedSource {
    RoundSeedSource::with_entropy(move |bytes| {
        bytes.fill(arena_seed);
        Ok(())
    })
}

fn cpu_provenance(case: EvaluationCase) -> Vec<(SeatId, CpuDifficulty, [u8; 32])> {
    let cpu_seeds = case.cpu_seeds();
    CPU_SEATS
        .into_iter()
        .enumerate()
        .map(|(index, seat)| (seat, case.difficulty, cpu_seeds[index]))
        .collect()
}

fn run_with_recording(
    case: EvaluationCase,
    tick_cap: usize,
    mut shadow: impl EvaluationRecording,
    navigation: MobNavigationPolicy,
) -> Result<EpisodeEvidence, String> {
    let cpu_seeds = case.cpu_seeds();
    let arena_seed = case.arena_seed;
    start_round(&mut shadow)?;

    let mut runtime = shadow.runtime().clone();
    let room_instance = runtime.identity().registry_serial;
    runtime.enable_evaluation_trace_for_test();
    let notice = runtime
        .current_notice()
        .map_err(|error| format!("initial evaluation publication: {error:?}"))?;
    let mut accepted_cpu_actions = drain_trace(&mut runtime, &mut shadow)?;

    let planner = CpuPlanner::without_workers();
    let cpus = CPU_SEATS
        .into_iter()
        .enumerate()
        .map(|(index, seat)| {
            CpuController::new(
                planner.clone(),
                seat,
                room_instance,
                cpu_seeds[index],
                case.profile(),
            )
        })
        .collect::<Vec<_>>();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(move |bytes| {
        bytes.fill(arena_seed);
        Ok(())
    });
    let mut driver = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        cpus,
    );
    driver.publish(notice);

    let initial = driver.runtime.replay_inspection();
    let initial_state = initial
        .simulation
        .as_ref()
        .ok_or_else(|| "evaluation round did not start".to_owned())?
        .state
        .clone();
    assert_identity(&initial, case, navigation)?;
    let mut collector = MetricsCollector::new(&initial_state);
    let mut planning = PumpReport::default();
    let mut script = ScriptState::default();

    for _ in 0..tick_cap {
        if driver
            .runtime
            .state_for_test()
            .is_some_and(|state| state.result().is_some())
        {
            break;
        }
        script.apply(case.script, &mut driver.runtime)?;
        accepted_cpu_actions.merge(drain_trace(&mut driver.runtime, &mut shadow)?);
        merge_planning(&mut planning, planner.pump_for_test(usize::MAX));
        driver
            .tick(Instant::now())
            .map_err(|error| format!("evaluation driver tick: {error:?}"))?;
        accepted_cpu_actions.merge(drain_trace(&mut driver.runtime, &mut shadow)?);
        let state = driver.runtime.state_for_test().unwrap();
        let (_, report) = driver.runtime.last_step_trace_for_test().unwrap();
        collector.observe(
            state,
            report,
            &driver.cpus,
            grenadiers_core::cpu::WorldId {
                room_instance,
                physical_round: 1,
            },
        );
    }

    for cpu in &mut driver.cpus {
        cpu.cancel();
    }
    merge_planning(&mut planning, planner.pump_for_test(usize::MAX));
    let final_drain = planner.pump_for_test(usize::MAX);
    if final_drain.dequeued != 0 {
        return Err("planner queue remained non-empty after complete final drain".to_owned());
    }
    let outstanding = driver.cpus.iter().filter(|cpu| cpu.has_pending()).count();
    let actual = driver.runtime.replay_inspection();
    if actual.lifecycle.round_attempt() != 1 {
        return Err("evaluation advanced beyond the first physical round".to_owned());
    }
    let shadow_exact = shadow.runtime().replay_inspection() == actual;
    if !shadow_exact {
        return Err("retained shadow diverged from actual driver runtime".to_owned());
    }
    shadow.verify(&actual)?;

    let state = actual.simulation.as_ref().unwrap().state.clone();
    let terminal = state.result().is_some();
    let (events, mobility, block_delta_unattributed, intents) = collector.finish();
    let admissions = admissions(&driver.cpus);
    let rng_draws = rng_draws(&driver.cpus);
    let planner_times = planning.planner_times.clone();
    let queue = QueueMetrics {
        dequeued: planning.dequeued,
        delivered: planning.delivered,
        cancelled: planning.cancelled,
        failed: planning.failed,
        dropped: planning.dropped,
        total_work: planning.work.iter().map(|value| u64::from(*value)).sum(),
        max_work: planning.work.iter().copied().max().unwrap_or(0),
        outstanding,
    };
    if queue.dequeued != queue.delivered + queue.cancelled + queue.failed + queue.dropped {
        return Err("planner terminal accounting was incomplete".to_owned());
    }
    Ok(EpisodeEvidence {
        measurement: EpisodeMeasurement {
            deterministic: DeterministicMeasurement {
                label: case.label,
                difficulty: format!("{:?}", case.difficulty),
                profile_version: case.profile().policy.version(),
                bomb_capacity_drop_multiplier: actual
                    .lifecycle
                    .settings()
                    .bomb_capacity_drop_multiplier()
                    .value(),
                arena_seed: case.arena_seed,
                simulation_root_seed: case.simulation_root_seed,
                cpu_seed_set: case.cpu_seed_set,
                cpu_seed_fingerprints: cpu_seeds.map(|seed| seed[0]),
                script: case.script,
                running_updates: state.running_updates(),
                terminal,
                censored: !terminal,
                round_result: state.result().map(|result| format!("{result:?}")),
                physical_round: actual.lifecycle.round_attempt(),
                events,
                mobility,
                block_delta_unattributed,
                intents,
                admissions,
                cadence: cadence_metrics(&driver.cpus),
                queue,
                accepted_cpu_actions,
                rng_draws,
                shadow_exact,
                replay_exact: true,
            },
            planner_times,
        },
        initial_state,
        final_inspection: actual,
        cpu_seeds,
    })
}

pub(super) fn configuration(difficulty: CpuDifficulty) -> SpikeRoomConfiguration {
    SpikeRoomConfiguration::new(
        LairRoomConfiguration::new(
            RoomConfiguration::new(
                &[
                    SeatControl::Human,
                    SeatControl::Cpu,
                    SeatControl::Cpu,
                    SeatControl::Cpu,
                ],
                Some(difficulty),
                BoardSize::FifteenByThirteen,
            )
            .unwrap(),
            StormCloudConfig::Single,
            RespawnLairTuning::DEFAULT.as_legacy(),
        ),
        SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
    )
}

#[derive(Default)]
struct ScriptState {
    manual_id: u32,
}

impl ScriptState {
    fn apply(
        &mut self,
        script: ReferenceScript,
        runtime: &mut RosterRuntime,
    ) -> Result<(), String> {
        let state = runtime.state_for_test().unwrap();
        if !state.actors()[SeatId::One.index()].alive() {
            return Ok(());
        }
        let tick = state.running_updates();
        let epoch = runtime.lifecycle().round_attempt();
        if tick.is_multiple_of(12) {
            let movement = match script {
                ReferenceScript::FixedIdle => None,
                ReferenceScript::BombRetreat => Some(if (tick / 48).is_multiple_of(2) {
                    Direction::East
                } else {
                    Direction::West
                }),
            };
            runtime
                .action(SeatId::One, epoch, RosterAction::Movement(movement))
                .map_err(|error| format!("reference movement rejected: {error:?}"))?;
        }
        if script == ReferenceScript::BombRetreat && tick.is_multiple_of(90) {
            self.manual_id = self
                .manual_id
                .checked_add(1)
                .ok_or_else(|| "reference manual id overflow".to_owned())?;
            runtime
                .action(SeatId::One, epoch, RosterAction::Bomb(self.manual_id))
                .map_err(|error| format!("reference bomb rejected: {error:?}"))?;
        }
        Ok(())
    }
}

fn merge_planning(total: &mut PumpReport, next: PumpReport) {
    total.dequeued += next.dequeued;
    total.delivered += next.delivered;
    total.cancelled += next.cancelled;
    total.failed += next.failed;
    total.dropped += next.dropped;
    total.work.extend(next.work);
    total.planned_intents.extend(next.planned_intents);
    total
        .planned_intents_by_job
        .extend(next.planned_intents_by_job);
    total.planner_times.extend(next.planner_times);
}
