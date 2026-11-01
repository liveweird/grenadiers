use std::collections::BTreeSet;
use std::sync::Arc;

use grenadiers_core::{
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, MobNavigationPolicy, Position,
        RoundAssignment, SeatId, SimulationSeed, State, StormCloudConfig, World, fixed_arena,
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

const CPU: SeatId = SeatId::Three;
const TARGET: CellId = CellId::new(3, 7);

pub(super) struct Outcome {
    pub exercised_respawn_lairs: bool,
    pub planted: bool,
    pub target_destroyed: bool,
    pub survived: bool,
    pub max_work: u32,
}

pub(super) fn run(profile: CpuProfile, difficulty: CpuDifficulty) -> Outcome {
    let (mut runtime, state) = running_runtime_and_scene(difficulty);
    let notice = runtime.install_running_scene_for_test(state).unwrap();
    let planner = CpuPlanner::without_workers();
    let controller = CpuController::new(planner.clone(), CPU, 4_036, [36; 32], profile);
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(36);
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
    let mut planted = false;
    let mut exercised_respawn_lairs = false;
    let mut max_work = 0;
    let mut finish_at = None;

    for _ in 0..420 {
        let report = planner.pump_for_test(usize::MAX);
        assert_eq!(report.failed, 0);
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();
        exercised_respawn_lairs |= owner.cpus[0]
            .latest_observation_for_test()
            .is_some_and(|observation| observation.respawn_lairs().is_some());
        let state = owner.runtime.state_for_test().unwrap();
        if !planted
            && let Some(bomb) = state
                .world()
                .bombs()
                .values()
                .find(|bomb| bomb.owner() == EntityId(3))
        {
            planted = bomb.cell() == CellId::new(3, 8);
            finish_at = bomb
                .detonates_at()
                .and_then(|tick| tick.0.checked_add(state.rules().flame_ticks() + 1))
                .map(grenadiers_core::game::Tick);
        }
        if finish_at.is_some_and(|tick| state.tick() >= tick) {
            break;
        }
    }
    let state = owner.runtime.state_for_test().unwrap();
    Outcome {
        exercised_respawn_lairs,
        planted,
        target_destroyed: !state.world().arena().blocks().contains(&TARGET),
        survived: finish_at.is_some_and(|tick| state.tick() >= tick)
            && state.actor(CPU).unwrap().alive(),
        max_work,
    }
}

fn running_runtime_and_scene(difficulty: CpuDifficulty) -> (RosterRuntime, State) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Human, SeatControl::Cpu],
        Some(difficulty),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc036").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: 4_036,
            configuration,
        },
        settings,
        StormCloudConfig::Disabled,
        RespawnLairTuning::DEFAULT,
        SpikePitTuning::new(false, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(36);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Two] {
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
    let state = controlled_state(runtime.state_for_test().unwrap(), settings);
    (runtime, state)
}

fn controlled_state(basis: &State, settings: RoomSettings) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let blocks = [
        (3, 1),
        (5, 1),
        (3, 2),
        (5, 2),
        (7, 2),
        (5, 3),
        (6, 3),
        (7, 3),
        (8, 3),
        (1, 4),
        (5, 4),
        (9, 4),
        (2, 5),
        (3, 5),
        (4, 5),
        (6, 5),
        (7, 5),
        (8, 5),
        (10, 5),
        (3, 6),
        (5, 6),
        (7, 6),
        (9, 6),
        (11, 6),
        (2, 7),
        (3, 7),
        (4, 7),
        (6, 7),
        (7, 7),
        (8, 7),
        (10, 7),
        (11, 7),
        (12, 7),
        (5, 8),
        (9, 8),
        (5, 9),
        (6, 9),
        (7, 9),
        (8, 9),
        (9, 9),
        (3, 10),
        (5, 10),
        (7, 10),
        (9, 10),
        (11, 10),
        (5, 11),
        (9, 11),
    ]
    .into_iter()
    .map(|(column, row)| CellId::new(column, row))
    .collect::<BTreeSet<_>>();
    let arena = Arena::new(
        terrain,
        blocks,
        [CellId::new(1, 1), CellId::new(13, 11), CellId::new(13, 1)],
    )
    .unwrap();
    let actors = [
        actor(1, SeatId::One, CellId::new(1, 1), settings),
        actor(2, SeatId::Two, CellId::new(13, 1), settings),
        actor(3, CPU, CellId::new(3, 8), settings),
    ];
    State::new_at_tick_with_assignment_and_seed(
        *basis.rules(),
        World::new(arena),
        actors,
        basis.tick(),
        RoundAssignment::first(),
        SimulationSeed::new([36; 32]),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, cell: CellId, settings: RoomSettings) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::North,
        speed: settings.starting_speed(),
    }
}
