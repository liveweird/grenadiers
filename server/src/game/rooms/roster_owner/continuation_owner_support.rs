use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use grenadiers_core::{
    game::{
        ActorInit, Arena, CellId, Direction, EffectKind, EnabledItems, EntityId,
        MobNavigationPolicy, PickupKind, Position, RoundAssignment, SeatId, SimulationSeed, State,
        StormCloudConfig, Tick, World, fixed_arena,
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
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

pub(super) const CPUS: [SeatId; 2] = [SeatId::Three, SeatId::Four];
const INSTANCE: u64 = 4_445;
const STARTS: [CellId; 4] = [
    CellId::new(1, 1),
    CellId::new(13, 1),
    CellId::new(1, 9),
    CellId::new(13, 9),
];

pub(super) struct Outcome {
    pub short_fuse_collected: [bool; 2],
    pub births: [Option<(Tick, Tick)>; 2],
    pub saw_original_routes: [bool; 2],
    pub saw_route_withdrawal: [bool; 2],
    pub route_withdrawal_ticks: [Option<Tick>; 2],
    pub saw_continuation: [bool; 2],
    pub admission_ticks: [Option<Tick>; 2],
    pub first_movement_command_ticks: [Option<Tick>; 2],
    pub renewal_commits: [u32; 2],
    pub command_commits: [u32; 2],
    pub first_moves_after_admission: [Option<Tick>; 2],
    pub displacement_after_admission: [u32; 2],
    pub survived_flames: [bool; 2],
    pub max_work: u32,
    pub failed_work: usize,
}

pub(super) fn run(lairs: bool) -> Outcome {
    let (mut owner, planner) = driver(lairs);
    // The pickup is present in the installed scene. The normal owner/runtime
    // update, rather than fixture construction, performs both collections.
    owner.tick(Instant::now()).unwrap();
    let initial = owner.runtime.state_for_test().unwrap();
    assert_eq!(initial.rules().enabled_items(), EnabledItems::ALL);
    assert!(
        STARTS[2..]
            .iter()
            .all(|cell| !initial.world().exposed_pickups().contains_key(cell))
    );
    let short_fuse_collected = CPUS.map(|seat| {
        initial
            .actor(seat)
            .unwrap()
            .effects()
            .active(EffectKind::ShortFuse, initial.tick())
    });
    assert!(short_fuse_collected.into_iter().all(|value| value));

    let mut births = [None; 2];
    let mut saw_original_routes = [false; 2];
    let mut saw_route_withdrawal = [false; 2];
    let mut route_withdrawal_ticks = [None; 2];
    let mut saw_continuation = [false; 2];
    let mut previous_positions = CPUS.map(|seat| initial.actor(seat).unwrap().position());
    let mut admission_positions = [None; 2];
    let mut first_moves_after_admission = [None; 2];
    let mut displacement_after_admission = [0; 2];
    let mut finish_at = None;
    let mut max_work = 0;
    let mut failed_work = 0;

    for _ in 0..500 {
        let report = planner.pump_for_test(usize::MAX);
        failed_work += report.failed;
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        for (index, seat) in CPUS.into_iter().enumerate() {
            saw_original_routes[index] |= owner.cpus[index].has_committed_route();
            saw_continuation[index] |= owner.cpus[index].has_continuation_authority_for_test();
            let trace = owner.cpus[index].continuation_trace_for_test();
            let position = state.actor(seat).unwrap().position();
            if trace.admission_tick.is_some() && admission_positions[index].is_none() {
                admission_positions[index] = Some(previous_positions[index]);
            }
            if let Some(origin) = admission_positions[index] {
                let distance = origin.x.abs_diff(position.x) + origin.y.abs_diff(position.y);
                displacement_after_admission[index] =
                    displacement_after_admission[index].max(distance);
                if distance > 0 && first_moves_after_admission[index].is_none() {
                    first_moves_after_admission[index] = Some(state.tick());
                }
            }
            previous_positions[index] = position;
            if saw_original_routes[index]
                && !owner.cpus[index].has_committed_route()
                && state.world().bombs().len() == 2
            {
                saw_route_withdrawal[index] = true;
                route_withdrawal_ticks[index].get_or_insert(state.tick());
            }
            if births[index].is_none()
                && let Some(bomb) = state
                    .world()
                    .bombs()
                    .values()
                    .find(|bomb| bomb.owner() == EntityId(seat.index() as u32 + 1))
            {
                let detonation = bomb.detonates_at().unwrap();
                births[index] = Some((bomb.placed_at(), detonation));
                finish_at = Some(Tick(detonation.0 + state.rules().flame_ticks() + 1));
            }
        }
        if finish_at.is_some_and(|tick| state.tick() >= tick) {
            break;
        }
    }
    let state = owner.runtime.state_for_test().unwrap();
    let survived_flames = CPUS.map(|seat| {
        finish_at.is_some_and(|tick| state.tick() >= tick) && state.actor(seat).unwrap().alive()
    });
    let traces = [
        owner.cpus[0].continuation_trace_for_test(),
        owner.cpus[1].continuation_trace_for_test(),
    ];
    Outcome {
        short_fuse_collected,
        births,
        saw_original_routes,
        saw_route_withdrawal,
        route_withdrawal_ticks,
        saw_continuation,
        admission_ticks: traces.map(|trace| trace.admission_tick),
        first_movement_command_ticks: traces.map(|trace| trace.first_movement_command_tick),
        renewal_commits: traces.map(|trace| trace.renewal_commits),
        command_commits: traces.map(|trace| trace.command_commits),
        first_moves_after_admission,
        displacement_after_admission,
        survived_flames,
        max_work,
        failed_work,
    }
}

pub(super) fn driver(lairs: bool) -> (Driver, CpuPlanner) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let controls = [
        SeatControl::Human,
        SeatControl::Human,
        SeatControl::Cpu,
        SeatControl::Cpu,
    ];
    let configuration = RoomConfiguration::new(
        &controls,
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc445").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        StormCloudConfig::Disabled,
        RespawnLairTuning::new(lairs, RespawnLairSettings::DEFAULT),
        SpikePitTuning::new(false, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(45);
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
    let basis = runtime.state_for_test().unwrap();
    let controlled = controlled_state(basis, settings);
    let notice = runtime.install_running_scene_for_test(controlled).unwrap();
    let planner = CpuPlanner::without_workers();
    let cpus = [
        CpuController::new(
            planner.clone(),
            SeatId::Three,
            INSTANCE,
            [44; 32],
            CpuProfile::CONTINUATION_V44_STANDARD,
        ),
        CpuController::new(
            planner.clone(),
            SeatId::Four,
            INSTANCE,
            [45; 32],
            CpuProfile::HARD_V45,
        ),
    ]
    .into_iter()
    .collect();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(45);
        Ok(())
    });
    let mut owner = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        cpus,
    );
    owner.publish(notice);
    (owner, planner)
}

fn controlled_state(basis: &State, settings: RoomSettings) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        blocks(),
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap();
    let pickups = BTreeMap::from([
        (STARTS[2], PickupKind::ShortFuse),
        (STARTS[3], PickupKind::ShortFuse),
    ]);
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), pickups).unwrap();
    let actors = SeatId::ALL.map(|seat| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(STARTS[seat.index()]),
        heading: Direction::South,
        speed: settings.starting_speed(),
    });
    State::new_at_tick_with_assignment_and_seed(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        RoundAssignment::first(),
        SimulationSeed::new([35; 32]),
    )
    .unwrap()
}

fn blocks() -> BTreeSet<CellId> {
    const RAW: &[(u16, u16)] = &[
        (3, 1),
        (4, 1),
        (6, 1),
        (7, 1),
        (8, 1),
        (10, 1),
        (3, 2),
        (5, 2),
        (9, 2),
        (11, 2),
        (1, 3),
        (2, 3),
        (5, 3),
        (6, 3),
        (8, 3),
        (9, 3),
        (12, 3),
        (1, 4),
        (5, 4),
        (7, 4),
        (9, 4),
        (13, 4),
        (2, 5),
        (3, 5),
        (5, 5),
        (6, 5),
        (7, 5),
        (8, 5),
        (9, 5),
        (11, 5),
        (12, 5),
        (1, 6),
        (3, 6),
        (5, 6),
        (7, 6),
        (9, 6),
        (11, 6),
        (13, 6),
        (2, 7),
        (3, 7),
        (5, 7),
        (6, 7),
        (7, 7),
        (8, 7),
        (9, 7),
        (11, 7),
        (12, 7),
        (1, 8),
        (5, 8),
        (7, 8),
        (9, 8),
        (13, 8),
        (2, 9),
        (5, 9),
        (6, 9),
        (8, 9),
        (9, 9),
        (12, 9),
        (3, 10),
        (5, 10),
        (9, 10),
        (11, 10),
        (4, 11),
        (6, 11),
        (7, 11),
        (8, 11),
        (10, 11),
    ];
    RAW.iter()
        .map(|&(column, row)| CellId::new(column, row))
        .collect()
}
