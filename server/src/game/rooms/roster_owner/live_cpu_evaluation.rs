use std::sync::Arc;

#[path = "live_cpu_evaluation_identity.rs"]
mod identity;

use grenadiers_core::{
    game::{Direction, EnabledItems, SeatId, StormCloudConfig},
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuDifficulty, LairRoomConfiguration,
        RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings,
        SpikePitTuning, SpikeRoomConfiguration,
    },
};
use tokio::{sync::watch, time::Instant};

use super::{
    super::{
        RandomFill,
        cpu::{CpuController, CpuPlanner, PumpReport},
        replay_v39::{self, RecordingRosterRuntime, ReplayAction},
        roster_runtime::{EvaluationOperation, RosterAction, RosterRuntime},
    },
    RosterOwnerNotice,
    driver::Driver,
    live_cpu_evaluation_metrics::{MetricsCollector, admissions, cadence_metrics, rng_draws},
    live_cpu_evaluation_types::{
        AcceptedCpuActions, CPU_SEATS, DeterministicMeasurement, EpisodeEvidence,
        EpisodeMeasurement, EvaluationCase, QueueMetrics, ReferenceScript,
    },
};
use crate::round_seed::RoundSeedSource;
use identity::assert_identity;

pub(super) const FULL_ROUND_CAP: usize = 10_800;

pub(super) fn run(case: EvaluationCase, tick_cap: usize) -> Result<EpisodeEvidence, String> {
    let cpu_seeds = case.cpu_seeds();
    let configuration = configuration(case.difficulty);
    let settings = RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT);
    let arena_seed = case.arena_seed;
    let mut shadow = RecordingRosterRuntime::new_for_evaluation(
        configuration,
        settings,
        RoundSeedSource::with_entropy(move |bytes| {
            bytes.fill(arena_seed);
            Ok(())
        }),
        [case.simulation_root_seed; 32],
        CPU_SEATS
            .into_iter()
            .enumerate()
            .map(|(index, seat)| (seat, case.difficulty, cpu_seeds[index]))
            .collect(),
    )
    .map_err(|error| format!("construct Replay39 evaluation fixture: {error:?}"))?;
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
    assert_identity(&initial, case)?;
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
    let (_, journal) = shadow.into_parts();
    replay_v39::verify_exact(&journal, &actual)?;

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

pub(super) fn start_round(recording: &mut RecordingRosterRuntime) -> Result<(), String> {
    recording
        .admit(SeatId::One, false)
        .map_err(|error| format!("admit evaluation human: {error:?}"))?;
    recording
        .human_action(
            SeatId::One,
            1,
            ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .map_err(|error| format!("ready evaluation human: {error:?}"))?;
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recording
            .settle([None; 4])
            .map_err(|error| format!("evaluation countdown: {error:?}"))?;
    }
    recording
        .publish_flush()
        .map_err(|error| format!("initial evaluation flush: {error:?}"))
}

pub(super) fn drain_trace(
    actual: &mut RosterRuntime,
    shadow: &mut RecordingRosterRuntime,
) -> Result<AcceptedCpuActions, String> {
    let mut accepted = AcceptedCpuActions::default();
    let operations = actual
        .drain_evaluation_trace_for_test()
        .map_err(|error| format!("evaluation trace failed: {error:?}"))?;
    for operation in operations {
        match operation {
            EvaluationOperation::HumanAction {
                seat,
                epoch,
                action,
            } => shadow
                .human_action(seat, epoch, replay_action(action)?)
                .map_err(|error| format!("shadow human action: {error:?}"))?,
            EvaluationOperation::CpuAction {
                seat,
                epoch,
                action,
            } => {
                if let Some(index) = CPU_SEATS.iter().position(|candidate| *candidate == seat) {
                    accepted.movement[index] +=
                        u64::from(matches!(action, RosterAction::Movement(_)));
                    accepted.bombs[index] += u64::from(matches!(action, RosterAction::Bomb(_)));
                }
                shadow
                    .cpu_action(seat, epoch, replay_action(action)?)
                    .map_err(|error| format!("shadow CPU action: {error:?}"))?;
            }
            EvaluationOperation::InputFence(seat) => shadow.input_fence(seat),
            EvaluationOperation::Settle { tick, departures } => {
                if shadow.runtime().lifecycle().tick() != tick {
                    return Err("shadow settle pre-operation tick mismatch".to_owned());
                }
                shadow
                    .settle(departures)
                    .map_err(|error| format!("shadow settle: {error:?}"))?;
            }
            EvaluationOperation::PublicationFlush => shadow
                .publish_flush()
                .map_err(|error| format!("shadow publication flush: {error:?}"))?,
        }
    }
    Ok(accepted)
}

fn replay_action(action: RosterAction) -> Result<ReplayAction, String> {
    Ok(match action {
        RosterAction::Ready {
            ready,
            settings_revision,
        } => ReplayAction::Ready {
            ready,
            settings_revision,
        },
        RosterAction::Movement(direction) => ReplayAction::Movement(direction),
        RosterAction::Bomb(id) => ReplayAction::Bomb(id),
        RosterAction::Equipment(id) => ReplayAction::Equipment(id),
        RosterAction::Rematch(value) => ReplayAction::Rematch(value),
        RosterAction::UpdateSettings { .. } => {
            return Err("generic settings updates are outside live evaluation".to_owned());
        }
    })
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
