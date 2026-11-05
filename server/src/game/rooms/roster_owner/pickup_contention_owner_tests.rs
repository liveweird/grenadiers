//! Controlled public setup; no generated-world or restored-match provenance is implied.
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use grenadiers_core::{
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, MobNavigationPolicy,
        PickupKind, Position, RoundActorInit, SeatId, SimulationSeed, SpeedLevel, State,
        StepEventKind, StormCloudConfig, Tick, World, fixed_arena,
    },
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, BombCapacityDropMultiplier, CpuDifficulty, CpuProfile, MatchPhase,
        RespawnLairTuning, RoomConfiguration, RoomSettings, SeatControl, SpikePitTuning,
    },
};
use tokio::{sync::watch, time::Instant};

use super::{RosterOwnerNotice, driver::Driver};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime, SpikeRuntimeEnvironment},
    },
    round_seed::RoundSeedSource,
};

const CPU: SeatId = SeatId::Two;
const RIVAL: SeatId = SeatId::One;
const CPU_START: CellId = CellId::new(7, 7);
const ALTERNATIVE: CellId = CellId::new(5, 7);
const CONTESTED: CellId = CellId::new(9, 7);
const RIVAL_START: CellId = CellId::new(13, 7);

#[derive(Debug)]
struct Outcome {
    initial_pickups: BTreeMap<CellId, PickupKind>,
    cpu_collection: Option<(Tick, EntityId, CellId, PickupKind)>,
    rival_collection: Option<(Tick, EntityId, CellId, PickupKind)>,
    alternative_present_at_contested_collection: bool,
    cpu_progress_after_collection: bool,
    delivered: usize,
    failed: usize,
    max_work: u32,
}

fn run(profile: CpuProfile) -> Outcome {
    let (mut owner, planner) = driver(profile);
    let epoch = owner.runtime.lifecycle().round_attempt();
    owner
        .runtime
        .action(RIVAL, epoch, RosterAction::Movement(Some(Direction::West)))
        .unwrap();
    let initial = owner.runtime.state_for_test().unwrap();
    let initial_pickups = initial
        .world()
        .exposed_pickups()
        .iter()
        .map(|(&cell, pickup)| (cell, pickup.kind()))
        .collect();
    let mut cpu_collection = None;
    let mut rival_collection = None;
    let mut cpu_collection_cell = None;
    let mut alternative_present_at_contested_collection = false;
    let mut cpu_progress_after_collection = false;
    let mut delivered = 0;
    let mut failed = 0;
    let mut max_work = 0;
    let mut rival_stopped = false;

    for update in 0..180 {
        let contested_remains = owner
            .runtime
            .state_for_test()
            .unwrap()
            .world()
            .exposed_pickups()
            .contains_key(&CONTESTED);
        if contested_remains && update % 10 == 0 {
            owner
                .runtime
                .action(RIVAL, epoch, RosterAction::Movement(Some(Direction::West)))
                .unwrap();
        } else if !contested_remains && !rival_stopped {
            owner
                .runtime
                .action(RIVAL, epoch, RosterAction::Movement(None))
                .unwrap();
            rival_stopped = true;
        }
        let report = planner.pump_for_test(usize::MAX);
        delivered += report.delivered;
        failed += report.failed;
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        let (_, physics) = owner.runtime.last_step_trace_for_test().unwrap();
        for event in &physics.events {
            if let StepEventKind::PickupCollected {
                id,
                seat,
                cell,
                kind,
            } = &event.kind
            {
                if *seat == CPU && cpu_collection.is_none() {
                    cpu_collection = Some((event.tick, *id, *cell, *kind));
                    cpu_collection_cell = state.actor(CPU).unwrap().position().cell();
                    if *cell == CONTESTED {
                        alternative_present_at_contested_collection =
                            state.world().exposed_pickups().contains_key(&ALTERNATIVE);
                    }
                } else if *seat == RIVAL && rival_collection.is_none() {
                    rival_collection = Some((event.tick, *id, *cell, *kind));
                }
            }
        }
        cpu_progress_after_collection |= cpu_collection_cell.is_some_and(|cell| {
            state
                .actor(CPU)
                .unwrap()
                .position()
                .cell()
                .is_some_and(|current| current != cell)
        });
        if cpu_collection.is_some()
            && (rival_collection.is_some() || profile == CpuProfile::HARD_V66)
            && cpu_progress_after_collection
        {
            break;
        }
    }
    assert_eq!(owner.runtime.lifecycle().phase(), MatchPhase::Running);
    Outcome {
        initial_pickups,
        cpu_collection,
        rival_collection,
        alternative_present_at_contested_collection,
        cpu_progress_after_collection,
        delivered,
        failed,
        max_work,
    }
}

fn driver(profile: CpuProfile) -> (Driver, CpuPlanner) {
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let settings = RoomSettings::DEFAULT
        .with_enabled_items(EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(BombCapacityDropMultiplier::DEFAULT);
    let mut runtime = RosterRuntime::new_spike_v21_pickup_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc066").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: 4_066,
            configuration,
        },
        settings,
        SpikeRuntimeEnvironment {
            cloud: StormCloudConfig::Disabled,
            lairs: RespawnLairTuning::DEFAULT,
            spike_pits: SpikePitTuning::DEFAULT,
            navigation: MobNavigationPolicy::TimedEscapeV3,
        },
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(66);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([66; 32]))),
    )
    .unwrap();
    let admission = runtime.prepare_join(RIVAL).unwrap();
    runtime.commit_admission(admission);
    runtime
        .action(
            RIVAL,
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
    let controlled = controlled_scene(runtime.state_for_test().unwrap());
    let notice = runtime.install_running_scene_for_test(controlled).unwrap();
    let planner = CpuPlanner::without_workers();
    let controller = CpuController::new(planner.clone(), CPU, 4_066, [66; 32], profile);
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(66);
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

#[test]
fn queued_hard66_collects_the_winnable_contested_pickup_before_the_row_major_alternative() {
    let hard65 = run(CpuProfile::HARD_V65);
    let hard66 = run(CpuProfile::HARD_V66);
    eprintln!("HARD66_PICKUP_WITNESS hard65={hard65:?} hard66={hard66:?}");
    let expected = BTreeMap::from([
        (ALTERNATIVE, PickupKind::BlastRange),
        (CONTESTED, PickupKind::BlastRange),
    ]);

    assert_eq!(hard65.initial_pickups, expected);
    assert_eq!(hard66.initial_pickups, expected);
    assert_eq!(CPU_START.column.abs_diff(CONTESTED.column), 2);
    assert_eq!(RIVAL_START.column.abs_diff(CONTESTED.column), 4);
    assert!(matches!(
        hard65.cpu_collection,
        Some((_, EntityId(2), ALTERNATIVE, PickupKind::BlastRange))
    ));
    assert!(matches!(
        hard65.rival_collection,
        Some((_, EntityId(1), CONTESTED, PickupKind::BlastRange))
    ));
    assert!(matches!(
        hard66.cpu_collection,
        Some((_, EntityId(2), CONTESTED, PickupKind::BlastRange))
    ));
    assert!(hard66.alternative_present_at_contested_collection);
    assert!(hard66.cpu_progress_after_collection);
    for outcome in [&hard65, &hard66] {
        assert!(outcome.delivered > 0, "{outcome:?}");
        assert_eq!(outcome.failed, 0, "{outcome:?}");
        assert!(
            outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
            "{outcome:?}"
        );
    }
}

fn controlled_scene(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([
            (ALTERNATIVE, PickupKind::BlastRange),
            (CONTESTED, PickupKind::BlastRange),
        ]),
    )
    .unwrap();
    let actors = [
        actor(1, RIVAL, RIVAL_START, Direction::West),
        actor(2, CPU, CPU_START, Direction::East),
    ];
    State::new_round(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        basis.round_assignment(),
        Some(SimulationSeed::new([66; 32])),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, cell: CellId, heading: Direction) -> RoundActorInit {
    RoundActorInit {
        actor: ActorInit {
            id: EntityId(id),
            seat,
            position: Position::center(cell),
            heading,
            speed: SpeedLevel::Base,
        },
        participating: true,
    }
}
