use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use grenadiers_core::{
    cpu::{WorldId, policy::StrategicTarget},
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, InputAction,
        MobNavigationPolicy, PickupKind, Position, RoundAssignment, SeatId, SeatInput,
        SimulationSeed, SpeedLevel, State, StormCloudConfig, Tick, World, fixed_arena,
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

pub(super) const CPU: SeatId = SeatId::Two;
const RIVAL: SeatId = SeatId::Three;
const INSTANCE: u64 = 4_035;

pub(super) struct EpisodeResult {
    pub first_birth: Tick,
    pub first_detonation: Tick,
    pub second_birth: Option<Tick>,
    pub simultaneous_owned: usize,
    pub selected_rival: bool,
    pub rival_in_second_blast: bool,
    pub second_cell: Option<CellId>,
    pub cpu_at_second: Option<CellId>,
    pub rival_at_second: Option<CellId>,
    pub cpu_alive_after_blasts: bool,
    pub completed_blast_lifetimes: bool,
    pub retained_route_before_first_detonation: bool,
    pub max_work: u32,
}

pub(super) fn driver(profile: CpuProfile) -> (Driver, CpuPlanner) {
    let (mut runtime, controlled) = running_runtime_and_scene();
    let notice = runtime.install_running_scene_for_test(controlled).unwrap();
    let planner = CpuPlanner::without_workers();
    let controller = CpuController::new(planner.clone(), CPU, INSTANCE, [35; 32], profile);
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(35);
        Ok(())
    });
    let mut driver = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        vec![controller],
    );
    driver.publish(notice);
    (driver, planner)
}

pub(super) fn run(profile: CpuProfile) -> EpisodeResult {
    let (mut owner, planner) = driver(profile);
    let epoch = owner.runtime.lifecycle().round_attempt();
    let world = WorldId {
        room_instance: INSTANCE,
        physical_round: epoch,
    };
    let mut first: Option<Tick> = None;
    let mut second: Option<Tick> = None;
    let mut simultaneous_owned = 0;
    let mut selected_rival = false;
    let mut rival_in_second_blast = false;
    let mut second_cell = None;
    let mut cpu_at_second = None;
    let mut rival_at_second = None;
    let mut max_work = 0;
    let mut retained_route_before_first_detonation = false;
    let mut finish_at = None;
    let mut completed_blast_lifetimes = false;

    for _ in 0..900 {
        if let Some(first_birth) = first {
            let offset = owner.runtime.lifecycle().tick().0 - first_birth.0;
            let action = match offset {
                0 => Some(Some(Direction::East)),
                12 => Some(Some(Direction::North)),
                27 => Some(Some(Direction::North)),
                36 => Some(None),
                _ => None,
            };
            if let Some(direction) = action {
                owner
                    .runtime
                    .action(RIVAL, epoch, RosterAction::Movement(direction))
                    .unwrap();
            }
        }
        let report = planner.pump_for_test(usize::MAX);
        assert_eq!(report.failed, 0);
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        owner.tick(Instant::now()).unwrap();

        let state = owner.runtime.state_for_test().unwrap();
        let owned: Vec<_> = state
            .world()
            .bombs()
            .values()
            .filter(|bomb| bomb.owner() == EntityId(2))
            .copied()
            .collect();
        simultaneous_owned = simultaneous_owned.max(owned.len());
        if first.is_none()
            && let Some(bomb) = owned.first()
        {
            first = Some(bomb.placed_at());
            finish_at = Some(Tick(
                bomb.detonates_at().unwrap().0 + state.rules().flame_ticks() + 1,
            ));
        }
        if second.is_none()
            && let Some(first_birth) = first
            && let Some(bomb) = owned.iter().find(|bomb| bomb.placed_at() != first_birth)
        {
            second = Some(bomb.placed_at());
            finish_at = Some(Tick(
                bomb.detonates_at().unwrap().0 + state.rules().flame_ticks() + 1,
            ));
            selected_rival = owner.cpus[0]
                .intent_debug_summary_for(world)
                .is_some_and(|summary| {
                    summary.target == StrategicTarget::Opponent { seat: RIVAL }
                        && state.actor(RIVAL).unwrap().alive()
                });
            let rival = state.actor(RIVAL).unwrap().position().cell().unwrap();
            second_cell = Some(bomb.cell());
            cpu_at_second = state.actor(CPU).unwrap().position().cell();
            rival_at_second = Some(rival);
            let column = bomb.cell().column.abs_diff(rival.column);
            let row = bomb.cell().row.abs_diff(rival.row);
            rival_in_second_blast =
                (column == 0 && row <= bomb.range()) || (row == 0 && column <= bomb.range());
        }
        if first.is_some_and(|birth| state.tick().0 + 1 == birth.0 + 240) {
            retained_route_before_first_detonation = owner.cpus[0].has_committed_route();
        }
        if finish_at.is_some_and(|tick| state.tick() >= tick) {
            completed_blast_lifetimes = true;
            break;
        }
    }

    let state = owner.runtime.state_for_test().unwrap();
    let first_birth = first.unwrap();
    EpisodeResult {
        first_birth,
        first_detonation: Tick(first_birth.0 + state.rules().bomb_fuse_ticks()),
        second_birth: second,
        simultaneous_owned,
        selected_rival,
        rival_in_second_blast,
        second_cell,
        cpu_at_second,
        rival_at_second,
        cpu_alive_after_blasts: state.actor(CPU).unwrap().alive(),
        completed_blast_lifetimes,
        retained_route_before_first_detonation,
        max_work,
    }
}

pub(super) fn running_runtime_and_scene() -> (RosterRuntime, State) {
    let settings = RoomSettings::new(SpeedLevel::Four, 256, 3, 240)
        .unwrap()
        .with_enabled_items(EnabledItems::ALL);
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu, SeatControl::Human],
        Some(CpuDifficulty::Hard),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc035").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        StormCloudConfig::Disabled,
        RespawnLairTuning::new(false, RespawnLairSettings::DEFAULT),
        SpikePitTuning::new(false, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(35);
            Ok(())
        }),
        Arc::new(|epoch| Ok(SimulationSeed::new([epoch as u8; 32]))),
        MobNavigationPolicy::TimedEscapeV3,
    )
    .unwrap();
    for seat in [SeatId::One, SeatId::Three] {
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
    let mut controlled = controlled_state(basis, settings);
    for _ in 0..48 {
        controlled
            .step(&[SeatInput {
                seat: CPU,
                action: InputAction::Press(Direction::North),
            }])
            .unwrap();
        runtime.settle_boundary([None; 4]).unwrap();
    }
    controlled
        .step(&[SeatInput {
            seat: CPU,
            action: InputAction::Clear,
        }])
        .unwrap();
    runtime.settle_boundary([None; 4]).unwrap();
    assert_eq!(
        controlled.actor(CPU).unwrap().position().cell(),
        Some(CellId::new(5, 5))
    );
    assert_eq!(controlled.actor(CPU).unwrap().bomb_capacity(), 2);
    assert_eq!(controlled.actor(CPU).unwrap().bomb_range(), 4);
    (runtime, controlled)
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
        BTreeSet::new(),
        [CellId::new(1, 1), CellId::new(13, 11), CellId::new(13, 1)],
    )
    .unwrap();
    let pickups = BTreeMap::from([
        (CellId::new(5, 8), PickupKind::BlastRange),
        (CellId::new(5, 7), PickupKind::BlastRange),
        (CellId::new(5, 6), PickupKind::BombCapacity),
    ]);
    let world = World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), pickups).unwrap();
    let actors = [
        actor(
            1,
            SeatId::One,
            CellId::new(13, 11),
            Direction::West,
            settings,
        ),
        actor(2, CPU, CellId::new(5, 9), Direction::North, settings),
        actor(3, RIVAL, CellId::new(8, 5), Direction::East, settings),
    ];
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

fn actor(
    id: u32,
    seat: SeatId,
    cell: CellId,
    heading: Direction,
    settings: RoomSettings,
) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading,
        speed: settings.starting_speed(),
    }
}
