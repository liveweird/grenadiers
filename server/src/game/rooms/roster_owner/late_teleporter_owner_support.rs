use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use grenadiers_core::{
    cpu::{WorldId, policy::IntentDebugSummary},
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, MobNavigationPolicy,
        PickupKind, Position, RoundActorInit, SeatId, SimulationSeed, State, StormCloudConfig,
        Tick, World, fixed_arena,
    },
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, RespawnLairSettings, RespawnLairTuning,
        RoomConfiguration, RoomSettings, SeatControl, SpikePitSettings, SpikePitTuning,
    },
};
use tokio::{sync::watch, time::Instant};

use super::{RosterOwnerNotice, driver::Driver};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{AdmissionTrace, CpuController, CpuPlanner, PumpReport},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

// This is a controlled production-owner scene, not a restored match. It
// preserves sample 88's public roster poses, arena blocks, pickup and fixed
// lair/pit geometry. The seed, tick history, Three's departure cause and the
// evolving cloud/mob state are explicit fixture assumptions.
pub(super) const CPU: SeatId = SeatId::Four;
pub(super) const TELEPORTER: CellId = CellId::new(3, 2);
const INSTANCE: u64 = 5_461;

#[derive(Debug)]
pub(super) struct Sample {
    pub tick: Tick,
    pub position: Position,
    pub intent: Option<IntentDebugSummary>,
    pub controller: String,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub initial_position: Position,
    pub samples: Vec<Sample>,
    pub worker: PumpReport,
    pub admission: AdmissionTrace,
    pub teleporter_trace: Vec<(u8, Tick, Tick, bool)>,
    pub teleporter_remaining: bool,
    pub collection_tick: Option<Tick>,
    pub plant_tick: Option<Tick>,
    pub destroyed_block: bool,
    pub cpu_alive: bool,
    pub initial_block_count: usize,
    pub dead_three: bool,
    pub initial_storm_slow: Vec<Option<u16>>,
    pub has_lairs: bool,
    pub has_pits: bool,
    pub structural_environment_geometry: bool,
}

pub(super) fn run(profile: CpuProfile, ticks: usize) -> Outcome {
    let (mut owner, planner) = driver(profile);
    let world = WorldId {
        room_instance: INSTANCE,
        physical_round: 1,
    };
    let initial_position = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .position();
    let initial_observation = owner.cpus[0].latest_observation_for_test().unwrap();
    let mut samples = Vec::with_capacity(ticks);
    let mut aggregate = PumpReport::default();
    let mut collection_tick = None;
    let mut plant_tick = None;
    let initial_blocks = owner
        .runtime
        .state_for_test()
        .unwrap()
        .world()
        .arena()
        .blocks()
        .clone();
    for _ in 0..ticks {
        merge(&mut aggregate, planner.pump_for_test(usize::MAX));
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        samples.push(Sample {
            tick: state.tick(),
            position: state.actor(CPU).unwrap().position(),
            intent: owner.cpus[0].intent_debug_summary_for(world),
            controller: owner.cpus[0].controller_state_for_test(),
        });
        if collection_tick.is_none() && !state.world().exposed_pickups().contains_key(&TELEPORTER) {
            collection_tick = Some(state.tick());
        }
        if plant_tick.is_none()
            && let Some(bomb) = state
                .world()
                .bombs()
                .values()
                .find(|bomb| bomb.owner() == EntityId(4))
        {
            plant_tick = Some(bomb.placed_at());
        }
    }
    let state = owner.runtime.state_for_test().unwrap();
    Outcome {
        initial_position,
        samples,
        worker: aggregate,
        admission: owner.cpus[0].admission_trace_for_test(),
        teleporter_trace: owner.teleporter_trace_for_test(),
        teleporter_remaining: state.world().exposed_pickups().contains_key(&TELEPORTER),
        collection_tick,
        plant_tick,
        destroyed_block: initial_blocks
            .iter()
            .any(|cell| !state.world().arena().blocks().contains(cell)),
        cpu_alive: state.actor(CPU).unwrap().alive(),
        initial_block_count: initial_observation.parts().arena.blocks.len(),
        dead_three: matches!(
            &initial_observation.parts().actors[SeatId::Three.index()],
            grenadiers_core::cpu::ObservedActor::Visible(actor) if !actor.alive
        ),
        initial_storm_slow: initial_observation
            .environment()
            .unwrap()
            .actor_storm_slow_remaining_ticks()
            .to_vec(),
        has_lairs: initial_observation.respawn_lairs().is_some(),
        has_pits: initial_observation.spike_pits().is_some(),
        structural_environment_geometry: initial_observation.respawn_lairs().is_some_and(|lairs| {
            lairs
                .lairs()
                .iter()
                .map(|lair| lair.cell)
                .eq([CellId::new(6, 6), CellId::new(8, 6)])
        }) && initial_observation.spike_pits().is_some_and(
            |pits| {
                pits.pits().iter().map(|pit| pit.cell).eq([
                    CellId::new(6, 3),
                    CellId::new(8, 3),
                    CellId::new(6, 9),
                    CellId::new(8, 9),
                ])
            },
        ),
    }
}

fn driver(profile: CpuProfile) -> (Driver, CpuPlanner) {
    scenario_driver(profile, CPU, controlled_scene, Some(SeatId::Three))
}

pub(super) fn scenario_driver(
    profile: CpuProfile,
    cpu: SeatId,
    scene: fn(&State) -> State,
    departure: Option<SeatId>,
) -> (Driver, CpuPlanner) {
    scenario_driver_with_cpu_seed(profile, cpu, scene, departure, [47; 32])
}

pub(super) fn scenario_driver_with_cpu_seed(
    profile: CpuProfile,
    cpu: SeatId,
    scene: fn(&State) -> State,
    departure: Option<SeatId>,
    cpu_seed: [u8; 32],
) -> (Driver, CpuPlanner) {
    scenario_driver_for_cpus(profile, &[cpu], scene, departure, cpu_seed)
}

pub(super) fn scenario_driver_for_cpus(
    profile: CpuProfile,
    cpus: &[SeatId],
    scene: fn(&State) -> State,
    departure: Option<SeatId>,
    cpu_seed: [u8; 32],
) -> (Driver, CpuPlanner) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let mut controls = [SeatControl::Human; 4];
    for &cpu in cpus {
        controls[cpu.index()] = SeatControl::Cpu;
    }
    let configuration = RoomConfiguration::new(
        &controls,
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789ab5461").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        StormCloudConfig::Single,
        RespawnLairTuning::new(true, RespawnLairSettings::DEFAULT),
        SpikePitTuning::new(true, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(47);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([47; 32]))),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap();
    for seat in SeatId::ALL.into_iter().filter(|seat| !cpus.contains(seat)) {
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

fn controlled_scene(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    #[rustfmt::skip]
    let blocks = [(5,1),(5,2),(7,2),(4,3),(7,3),(3,4),(5,4),(7,4),(9,4),
        (2,5),(3,5),(5,5),(6,5),(7,5),(8,5),(9,5),(11,5),(12,5),(1,6),(3,6),
        (7,6),(11,6),(13,6),(1,7),(2,7),(3,7),(5,7),(6,7),(7,7),(8,7),(9,7),
        (11,7),(12,7),(3,8),(5,8),(7,8),(9,8),(11,8),(4,9),(7,9),(10,9),(3,10),
        (5,10),(7,10),(9,10),(4,11),(5,11),(9,11)]
        .into_iter().map(|(c,r)| CellId::new(c,r)).collect::<BTreeSet<_>>();
    let arena = Arena::new(
        terrain,
        blocks,
        [
            CellId::new(1, 1),
            CellId::new(13, 11),
            CellId::new(13, 1),
            CellId::new(1, 11),
        ],
    )
    .unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(TELEPORTER, PickupKind::Teleporter)]),
    )
    .unwrap();
    let actors = [
        actor(
            1,
            SeatId::One,
            Position {
                x: 11_776,
                y: 10_243,
            },
            Direction::East,
            true,
        ),
        actor(
            2,
            SeatId::Two,
            Position { x: 3_314, y: 9_728 },
            Direction::West,
            true,
        ),
        actor(
            3,
            SeatId::Three,
            Position { x: 8_667, y: 3_584 },
            Direction::West,
            true,
        ),
        actor(
            4,
            CPU,
            Position { x: 1_536, y: 2_724 },
            Direction::South,
            true,
        ),
    ];
    State::new_round(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        basis.round_assignment(),
        Some(SimulationSeed::new([47; 32])),
    )
    .unwrap()
}

fn actor(
    id: u32,
    seat: SeatId,
    position: Position,
    heading: Direction,
    participating: bool,
) -> RoundActorInit {
    RoundActorInit {
        actor: ActorInit {
            id: EntityId(id),
            seat,
            position,
            heading,
            speed: grenadiers_core::game::SpeedLevel::Base,
        },
        participating,
    }
}

fn merge(total: &mut PumpReport, next: PumpReport) {
    total.dequeued += next.dequeued;
    total.delivered += next.delivered;
    total.cancelled += next.cancelled;
    total.failed += next.failed;
    total.dropped += next.dropped;
    total.work.extend(next.work);
    total.planned_intents.extend(next.planned_intents);
    total.planner_times.extend(next.planner_times);
}
