use std::{collections::BTreeSet, sync::Arc};

use grenadiers_core::{
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, MobNavigationPolicy, Position,
        RoundAssignment, SeatId, SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World,
        fixed_arena,
    },
    protocol::{RoomCode, RoomId},
    room::{
        BoardSize, CpuDifficulty, CpuProfile, RespawnLairTuning, RoomConfiguration, RoomSettings,
        SeatControl, SpikePitSettings, SpikePitTuning,
    },
};
use tokio::{sync::watch, time::Instant};

use super::{RosterOwnerNotice, driver::Driver};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

pub(super) const CPU: SeatId = SeatId::Two;
const TARGET: CellId = CellId::new(9, 7);
const PLANT: CellId = CellId::new(10, 7);
const INSTANCE: u64 = 4_041;
const NATURAL_SPAWN_UPDATES: u64 = 600;

pub(super) struct Outcome {
    pub witness_start: Tick,
    pub planted_at: Option<Tick>,
    pub target_destroyed: bool,
    pub survived: bool,
    pub observed_typed_contact: bool,
    pub observed_blocked_birth_contact: bool,
    pub max_work: u32,
}

pub(super) fn run(profile: CpuProfile, difficulty: CpuDifficulty) -> Outcome {
    let (mut owner, planner) = driver_at_witness(profile, difficulty);
    let initial = owner.runtime.state_for_test().unwrap();
    assert_natural_witness(initial);
    let witness_start = initial.tick();
    let mut planted_at = None;
    let mut finish_at = None;
    let mut max_work = 0;
    let mut observed_typed_contact = false;
    let mut observed_blocked_birth_contact = false;

    for _ in 0..1_200 {
        let report = planner.pump_for_test(usize::MAX);
        assert_eq!(report.failed, 0);
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();
        observed_typed_contact |= owner.cpus[0].has_committed_topology_contact();
        observed_blocked_birth_contact |= owner.cpus[0].has_committed_blocked_birth_contact();
        let state = owner.runtime.state_for_test().unwrap();
        if planted_at.is_none()
            && let Some(bomb) = state
                .world()
                .bombs()
                .values()
                .find(|bomb| bomb.owner() == EntityId(2) && bomb.cell() == PLANT)
        {
            planted_at = Some(bomb.placed_at());
            finish_at = bomb
                .detonates_at()
                .and_then(|tick| tick.0.checked_add(state.rules().flame_ticks() + 1))
                .map(Tick);
        }
        if finish_at.is_some_and(|tick| state.tick() >= tick) {
            break;
        }
    }
    let state = owner.runtime.state_for_test().unwrap();
    Outcome {
        witness_start,
        planted_at,
        target_destroyed: !state.world().arena().blocks().contains(&TARGET),
        survived: finish_at.is_some_and(|tick| state.tick() >= tick)
            && state.actor(CPU).unwrap().alive(),
        observed_typed_contact,
        observed_blocked_birth_contact,
        max_work,
    }
}

pub(super) fn driver_at_witness(
    profile: CpuProfile,
    difficulty: CpuDifficulty,
) -> (Driver, CpuPlanner) {
    let (mut runtime, state) = running_runtime_and_scene(difficulty);
    let notice = runtime.install_running_scene_for_test(state).unwrap();
    let planner = CpuPlanner::without_workers();
    let controller = CpuController::new(planner.clone(), CPU, INSTANCE, [136; 32], profile);
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(136);
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

fn running_runtime_and_scene(difficulty: CpuDifficulty) -> (RosterRuntime, State) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(difficulty),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc041").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        StormCloudConfig::Disabled,
        RespawnLairTuning::DEFAULT,
        SpikePitTuning::new(false, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(136);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([136; 32]))),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap();
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
    let basis = runtime.state_for_test().unwrap();
    let mut controlled = controlled_state(basis, settings);
    for _ in 0..NATURAL_SPAWN_UPDATES {
        controlled.step(&[]).unwrap();
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_natural_witness(&controlled);
    (runtime, controlled)
}

fn assert_natural_witness(state: &State) {
    assert_eq!(state.running_updates(), NATURAL_SPAWN_UPDATES);
    assert_eq!(
        state.actor(CPU).unwrap().position(),
        Position::center(PLANT)
    );
    let mobs = state.mobs();
    assert!(
        !mobs.is_empty(),
        "default 10s lairs did not spawn naturally"
    );
    assert!(
        mobs.iter()
            .all(|mob| mob.position.cell() == Some(CellId::new(7, 6)))
    );
}

fn controlled_state(basis: &State, settings: RoomSettings) -> State {
    // This is the reviewed two-actor contact witness, not a claim that hidden
    // facts from any four-player capture were restored.
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let blocks = shared_blocks();
    let arena = Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap();
    let actors = [
        actor(1, SeatId::One, CellId::new(1, 1), settings),
        actor(2, CPU, PLANT, settings),
    ];
    State::new_at_tick_with_assignment_and_seed(
        *basis.rules(),
        World::new(arena),
        actors,
        basis.tick(),
        RoundAssignment::first(),
        SimulationSeed::new([136; 32]),
    )
    .unwrap()
}

fn shared_blocks() -> BTreeSet<CellId> {
    let raw = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core/tests/fixtures/trapped-mob-blocks.json"
    ));
    serde_json::from_str::<Vec<[u16; 2]>>(raw)
        .unwrap()
        .into_iter()
        .map(|[column, row]| CellId::new(column, row))
        .collect()
}

fn actor(id: u32, seat: SeatId, cell: CellId, _settings: RoomSettings) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::East,
        speed: SpeedLevel::Base,
    }
}
