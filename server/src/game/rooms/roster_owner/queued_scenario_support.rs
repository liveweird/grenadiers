//! Controlled real-owner scenarios; no generated-world or restored-match provenance is implied.
use super::{RosterOwnerNotice, driver::Driver};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};
use grenadiers_core::{
    game::StormCloudConfig,
    game::{EnabledItems, MobNavigationPolicy, SeatId, SimulationSeed, State},
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuProfile, RespawnLairSettings, RespawnLairTuning, RoomConfiguration,
        RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
    },
};
use std::sync::Arc;
use tokio::{sync::watch, time::Instant};
const INSTANCE: u64 = 5_461;
#[derive(Clone, Copy)]
pub(super) enum Environment {
    Active,
    Disabled,
}

pub(super) fn driver(
    profile: CpuProfile,
    cpus: &[SeatId],
    scene: fn(&State) -> State,
    departure: Option<SeatId>,
    cpu_seed: [u8; 32],
    environment: Environment,
) -> (Driver, CpuPlanner) {
    let mut controls = [SeatControl::Human; 4];
    for &cpu in cpus {
        controls[cpu.index()] = SeatControl::Cpu;
    }
    configured_driver(
        profile,
        cpus,
        &controls,
        scene,
        departure,
        cpu_seed,
        environment,
    )
}

pub(super) fn duel_driver(
    profile: CpuProfile,
    scene: fn(&State) -> State,
    cpu_seed: [u8; 32],
    environment: Environment,
) -> (Driver, CpuPlanner) {
    configured_driver(
        profile,
        &[SeatId::Two],
        &[SeatControl::Human, SeatControl::Cpu],
        scene,
        None,
        cpu_seed,
        environment,
    )
}

fn configured_driver(
    profile: CpuProfile,
    cpus: &[SeatId],
    controls: &[SeatControl],
    scene: fn(&State) -> State,
    departure: Option<SeatId>,
    cpu_seed: [u8; 32],
    environment: Environment,
) -> (Driver, CpuPlanner) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let configuration = RoomConfiguration::new(
        controls,
        Some(profile.difficulty),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let (cloud, lairs, pits) = match environment {
        Environment::Active => (
            StormCloudConfig::Single,
            RespawnLairTuning::new(true, RespawnLairSettings::DEFAULT),
            SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
        ),
        Environment::Disabled => (
            StormCloudConfig::Disabled,
            RespawnLairTuning::new(false, RespawnLairSettings::DEFAULT),
            SpikePitTuning::new(false, SpikePitSettings::DEFAULT),
        ),
    };
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789ab5461").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        cloud,
        lairs,
        pits,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(47);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([47; 32]))),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap();
    for seat in SeatId::ALL
        .into_iter()
        .take(controls.len())
        .filter(|seat| !cpus.contains(seat))
    {
        let admission = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(admission);
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let basis = runtime.state_for_test().unwrap();
    let scene = scene(basis);
    let scene_notice = runtime.install_running_scene_for_test(scene).unwrap();
    let departures = std::array::from_fn(|index| {
        (departure.is_some_and(|seat| seat.index() == index))
            .then_some(grenadiers_core::room::DepartureReason::Explicit)
    });
    let notice = if departure.is_some() {
        runtime.settle_boundary(departures).unwrap().unwrap()
    } else {
        scene_notice
    };
    let planner = CpuPlanner::without_workers();
    let controllers = cpus
        .iter()
        .map(|&cpu| CpuController::new(planner.clone(), cpu, INSTANCE, cpu_seed, profile))
        .collect();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(47);
        Ok(())
    });
    let mut owner = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        controllers,
    );
    owner.reset_teleporter_trace_for_test();
    owner.publish(notice);
    (owner, planner)
}
