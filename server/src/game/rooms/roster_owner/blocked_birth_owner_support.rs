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
        SeatControl, SpikePitTuning,
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
const TARGET: CellId = CellId::new(3, 5);
const PLANT: CellId = CellId::new(3, 6);
const INSTANCE: u64 = 4_243;

pub(super) struct Outcome {
    pub planted_at: Option<Tick>,
    pub target_destroyed: bool,
    pub survived: bool,
    pub blocked_birth_authority: bool,
    pub max_work: u32,
}

pub(super) fn run(profile: CpuProfile, difficulty: CpuDifficulty) -> Outcome {
    let (mut owner, planner) = driver(profile, difficulty);
    let mut planted_at = None;
    let mut finish_at = None;
    let mut blocked_birth_authority = false;
    let mut max_work = 0;
    for _ in 0..500 {
        let report = planner.pump_for_test(usize::MAX);
        assert_eq!(report.failed, 0);
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();
        blocked_birth_authority |= owner.cpus[0].has_committed_blocked_birth_contact();
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
        planted_at,
        target_destroyed: !state.world().arena().blocks().contains(&TARGET),
        survived: finish_at.is_some_and(|tick| state.tick() >= tick)
            && state.actor(CPU).unwrap().alive(),
        blocked_birth_authority,
        max_work,
    }
}

pub(super) fn driver(profile: CpuProfile, difficulty: CpuDifficulty) -> (Driver, CpuPlanner) {
    let (mut runtime, state) = running_runtime_and_scene(difficulty);
    let notice = runtime.install_running_scene_for_test(state).unwrap();
    let planner = CpuPlanner::without_workers();
    let controller = CpuController::new(planner.clone(), CPU, INSTANCE, [137; 32], profile);
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(137);
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
            id: RoomId::new("0123456789abcdef0123456789abc243").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        StormCloudConfig::Single,
        RespawnLairTuning::DEFAULT,
        SpikePitTuning::DEFAULT,
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(137);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([137; 32]))),
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
    let mut controlled = controlled_state(basis);
    for _ in 0..600 {
        controlled.step(&[]).unwrap();
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert_eq!(controlled.tick(), runtime.lifecycle().tick());
    assert_eq!(controlled.running_updates(), 600);
    assert!(controlled.mobs().is_empty());
    (runtime, controlled)
}

fn controlled_state(basis: &State) -> State {
    // This controlled active-environment witness shares the captured arena,
    // blocks, CPU pose, settings, and blocked lair births. Unlike public tick
    // 10741 it has two actors, CPU SeatTwo without Punch, a normal runtime
    // clock advanced exactly 600 updates, and the seed-137 cloud trajectory.
    // Cloud, respawn lairs, and spike pits remain enabled in both State and
    // Runtime; this is not a hidden-state or replay restoration claim.
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(terrain, blocks(), [CellId::new(1, 1), CellId::new(13, 11)]).unwrap();
    State::new_at_tick_with_assignment_and_seed(
        *basis.rules(),
        World::new(arena),
        [
            actor(1, SeatId::One, Position::center(CellId::new(1, 1))),
            actor(2, CPU, Position { x: 3_584, y: 6_974 }),
        ],
        basis.tick(),
        RoundAssignment::first(),
        SimulationSeed::new([137; 32]),
    )
    .unwrap()
}

fn blocks() -> BTreeSet<CellId> {
    serde_json::from_str::<Vec<[u16; 2]>>(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../core/tests/fixtures/blocked-birth-farm-blocks.json"
    )))
    .unwrap()
    .into_iter()
    .map(|[column, row]| CellId::new(column, row))
    .collect()
}

fn actor(id: u32, seat: SeatId, position: Position) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position,
        heading: Direction::South,
        speed: SpeedLevel::Base,
    }
}
