//! Generated public setup; no curated arena or restored-match provenance is implied.
use std::sync::Arc;

use grenadiers_core::{
    game::{EntityId, MobNavigationPolicy, SeatId, SimulationSeed, StormCloudConfig, Tick},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, RespawnLairTuning, RoomConfiguration, RoomSettings,
        SeatControl, SpikePitTuning,
    },
};
use tokio::{sync::watch, time::Instant};

use super::{RosterOwnerNotice, driver::Driver};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{CpuController, CpuPlanner, Easy68CadenceMetrics},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime, SpikeRuntimeEnvironment},
    },
    round_seed::RoundSeedSource,
};

const CPU: SeatId = SeatId::Two;
const INSTANCE: u64 = 4_068;

#[derive(Debug)]
pub(super) struct Outcome {
    pub profile: CpuProfile,
    pub moved: bool,
    pub confirmed_stop_at: Option<Tick>,
    pub resumed_after_pause_at: Option<Tick>,
    pub first_bypass_at: Option<Tick>,
    pub metrics: Easy68CadenceMetrics,
    pub dequeued: usize,
    pub delivered: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub dropped: usize,
    pub max_work: u32,
    pub queue_drained: bool,
    pub owned_bomb_seen: bool,
}

pub(super) fn run(profile: CpuProfile) -> Outcome {
    run_with(profile, false, 1_200)
}

pub(super) fn run_native() -> Outcome {
    run_with(CpuProfile::CERTIFIED_CADENCE_V68_EASY, true, 720)
}

fn run_with(profile: CpuProfile, native: bool, ticks: usize) -> Outcome {
    let (mut owner, planner) = driver(profile, native);
    let tick_interval = tick_interval(owner.runtime.state_for_test().unwrap().rules().tick_rate());
    let initial = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .position();
    let mut prior = initial;
    let mut moved = false;
    let mut confirmed_stop_at = None;
    let mut resumed_after_pause_at = None;
    let mut first_bypass_at = None;
    let mut completed_seen = false;
    let mut dequeued = 0;
    let mut delivered = 0;
    let mut failed = 0;
    let mut cancelled = 0;
    let mut dropped = 0;
    let mut max_work = 0;
    let mut owned_bomb_seen = false;

    for _ in 0..ticks {
        if native {
            std::thread::sleep(tick_interval);
        } else {
            account(
                planner.pump_for_test(usize::MAX),
                &mut dequeued,
                &mut delivered,
                &mut failed,
                &mut cancelled,
                &mut dropped,
                &mut max_work,
            );
        }
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        owned_bomb_seen |= state
            .world()
            .bombs()
            .values()
            .any(|bomb| bomb.owner() == EntityId(2));
        let position = state.actor(CPU).unwrap().position();
        moved |= position != initial;
        let metrics = owner.cpus[0].easy68_cadence_metrics_for_test();
        if metrics.planning_bypassed > 0 && first_bypass_at.is_none() {
            first_bypass_at = Some(state.tick());
        }
        if metrics.planning_deferred > 0
            && owner.cpu_stationary[CPU.index()]
            && confirmed_stop_at.is_none()
        {
            confirmed_stop_at = Some(state.tick());
        }
        completed_seen |= metrics.pause_completed > 0;
        if completed_seen
            && position.cell().is_some()
            && position.cell() != prior.cell()
            && resumed_after_pause_at.is_none()
        {
            resumed_after_pause_at = Some(state.tick());
        }
        prior = position;
    }
    for controller in &mut owner.cpus {
        controller.cancel();
    }
    let queue_drained = if native {
        std::thread::sleep(tick_interval);
        let before = planner.trace_for_test();
        std::thread::sleep(tick_interval);
        let report = planner.trace_for_test();
        let stable = before == report;
        account(
            report,
            &mut dequeued,
            &mut delivered,
            &mut failed,
            &mut cancelled,
            &mut dropped,
            &mut max_work,
        );
        stable
            && dequeued == delivered + failed + cancelled + dropped
            && owner
                .cpus
                .iter()
                .all(|controller| !controller.has_pending())
    } else {
        account(
            planner.pump_for_test(usize::MAX),
            &mut dequeued,
            &mut delivered,
            &mut failed,
            &mut cancelled,
            &mut dropped,
            &mut max_work,
        );
        planner.pump_for_test(usize::MAX).dequeued == 0
            && owner
                .cpus
                .iter()
                .all(|controller| !controller.has_pending())
    };
    Outcome {
        profile,
        moved,
        confirmed_stop_at,
        resumed_after_pause_at,
        first_bypass_at,
        metrics: owner.cpus[0].easy68_cadence_metrics_for_test(),
        dequeued,
        delivered,
        failed,
        cancelled,
        dropped,
        max_work,
        queue_drained,
        owned_bomb_seen,
    }
}

fn tick_interval(tick_rate: u16) -> std::time::Duration {
    std::time::Duration::from_nanos(1_000_000_000 / u64::from(tick_rate))
}

#[allow(clippy::too_many_arguments)]
fn account(
    report: crate::game::rooms::cpu::PumpReport,
    dequeued: &mut usize,
    delivered: &mut usize,
    failed: &mut usize,
    cancelled: &mut usize,
    dropped: &mut usize,
    max_work: &mut u32,
) {
    *dequeued += report.dequeued;
    *delivered += report.delivered;
    *failed += report.failed;
    *cancelled += report.cancelled;
    *dropped += report.dropped;
    *max_work = (*max_work).max(report.work.into_iter().max().unwrap_or(0));
}

fn driver(profile: CpuProfile, native: bool) -> (Driver, CpuPlanner) {
    let mut runtime = runtime();
    let admission = runtime.prepare_join(SeatId::One).unwrap();
    runtime.commit_admission(admission);
    runtime
        .action(
            SeatId::One,
            1,
            RosterAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let planner = if native {
        CpuPlanner::new().unwrap()
    } else {
        CpuPlanner::without_workers()
    };
    let controller = CpuController::new(planner.clone(), CPU, INSTANCE, [68; 32], profile);
    let notice = runtime.current_notice().unwrap();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(68);
        Ok(())
    });
    let mut owner = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        vec![controller],
    );
    owner.publish(notice);
    (owner, planner)
}

fn runtime() -> RosterRuntime {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Easy),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    RosterRuntime::new_spike_v21_easy_cadence_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc068").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        RoomSettings::DEFAULT,
        SpikeRuntimeEnvironment {
            cloud: StormCloudConfig::Single,
            lairs: RespawnLairTuning::DEFAULT,
            spike_pits: SpikePitTuning::DEFAULT,
            navigation: MobNavigationPolicy::TimedEscapeV3,
        },
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(68);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([68; 32]))),
    )
    .unwrap()
}
