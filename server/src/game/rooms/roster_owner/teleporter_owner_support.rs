use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use grenadiers_core::{
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, MobNavigationPolicy,
        PickupKind, Position, RoundAssignment, SeatId, SimulationSeed, State, StormCloudConfig,
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
        cpu::{CpuController, CpuPlanner},
        roster_runtime::{RosterAction, RosterIdentity, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

pub(super) const CPU: SeatId = SeatId::Two;
pub(super) const TELEPORTER: CellId = CellId::new(3, 9);
pub(super) const TARGET_BLOCK: CellId = CellId::new(11, 1);
const HARD_TARGET_BLOCK: CellId = CellId::new(2, 7);
const HARD_EAST_TARGET_BLOCK: CellId = CellId::new(4, 7);
const CPU_START: CellId = CellId::new(3, 10);
const HUMAN_START: CellId = CellId::new(13, 1);
const INSTANCE: u64 = 4_647;

#[derive(Debug)]
pub(super) struct Outcome {
    pub collection_tick: Option<Tick>,
    pub movement_admission_ticks: Vec<Tick>,
    pub movement_sealed_ticks: Vec<Tick>,
    pub movement_update_ticks: Vec<Tick>,
    pub stop_submission_ticks: Vec<Tick>,
    pub stop_sealed_ticks: Vec<Tick>,
    pub physical_stop_ticks: Vec<Tick>,
    pub stop_pose_held: Vec<bool>,
    pub stop_inputs_cleared: Vec<bool>,
    pub stopped_completion_tick: Option<Tick>,
    pub cpu_teleport_tick: Option<Tick>,
    pub human_teleport_tick: Option<Tick>,
    pub cpu_landed_at_human_pose: bool,
    pub human_landed_at_collection_endpoint: bool,
    pub plant_tick: Option<Tick>,
    pub plant_cell: Option<CellId>,
    pub plant_reaches_named_target: bool,
    pub named_target: CellId,
    pub named_target_existed: bool,
    pub target_destroyed: bool,
    pub survived_flames: bool,
    pub positions: Vec<(Tick, Position)>,
    pub max_work: u32,
    pub failed_work: usize,
    pub inputs: String,
}

pub(super) fn run(profile: CpuProfile, difficulty: CpuDifficulty, lairs: bool) -> Outcome {
    run_with_worker(profile, difficulty, lairs, false)
}

pub(super) fn run_native(profile: CpuProfile, difficulty: CpuDifficulty) -> Outcome {
    run_with_worker(profile, difficulty, false, true)
}

fn run_with_worker(
    profile: CpuProfile,
    difficulty: CpuDifficulty,
    lairs: bool,
    native: bool,
) -> Outcome {
    let (mut owner, planner) = test_driver(profile, difficulty, lairs, native);
    let mut collection_tick = None;
    let mut movement_updates = Vec::new();
    let mut stopped_completion_tick = None;
    let mut physical_stop_ticks = Vec::new();
    let mut stop_pose_held = Vec::new();
    let mut stop_inputs_cleared = Vec::new();
    let mut stop_seen = false;
    let mut active_pose = None;
    let mut cpu_teleport_tick = None;
    let mut human_teleport_tick = None;
    let mut cpu_landed_at_human_pose = false;
    let mut human_landed_at_collection_endpoint = false;
    let mut plant_tick = None;
    let mut plant_cell = None;
    let mut plant_reaches_named_target = false;
    let mut finish_at = None;
    let mut was_controlled = false;
    let mut positions = Vec::new();
    let mut max_work = 0;
    let mut failed_work = 0;
    let mut target_destroyed = false;
    let named_target = if difficulty == CpuDifficulty::Hard {
        HARD_TARGET_BLOCK
    } else {
        TARGET_BLOCK
    };
    let initial = owner.runtime.state_for_test().unwrap();
    let named_target_existed = initial.world().arena().blocks().contains(&named_target);
    let mut last_distinct_cpu = initial.actor(CPU).unwrap().position();
    let mut previous_distinct_cpu = last_distinct_cpu;
    let mut prior_human = initial.actor(SeatId::One).unwrap().position();
    for _ in 0..1_200 {
        if native {
            // Pace every simulated update, including idle updates, at the
            // production rules10 fixed tick interval. Planner latency remains
            // diagnostic rather than an accelerated freshness assumption.
            std::thread::sleep(std::time::Duration::from_nanos(1_000_000_000 / 60));
        } else {
            let report = planner.pump_for_test(usize::MAX);
            failed_work += report.failed;
            max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        }
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        target_destroyed |= !state.world().arena().blocks().contains(&named_target);
        let controlled = owner.cpus[0].teleporter_controlled();
        let position = state.actor(CPU).unwrap().position();
        if controlled && !was_controlled {
            movement_updates.push(state.tick());
            active_pose = Some(position);
            stop_seen = false;
        }
        if controlled && owner.cpu_stationary[CPU.index()] && !stop_seen {
            physical_stop_ticks.push(state.tick());
            stop_pose_held.push(active_pose == Some(position));
            stop_inputs_cleared.push(mailbox_is_cleared(
                &owner.runtime.replay_inspection().inputs,
            ));
            stop_seen = true;
        }
        if !controlled && was_controlled && collection_tick.is_some() {
            stopped_completion_tick.get_or_insert(state.tick());
        }
        was_controlled = controlled;
        if positions.last().is_none_or(|(_, prior)| *prior != position) {
            positions.push((state.tick(), position));
        }
        if collection_tick.is_none() && !state.world().exposed_pickups().contains_key(&TELEPORTER) {
            collection_tick = Some(state.tick());
            let cpu = state.actor(CPU).unwrap();
            let human = state.actor(SeatId::One).unwrap();
            cpu_teleport_tick = cpu.last_teleport_tick();
            human_teleport_tick = human.last_teleport_tick();
            cpu_landed_at_human_pose = cpu.position() == prior_human;
            let delta_x = last_distinct_cpu.x - previous_distinct_cpu.x;
            let delta_y = last_distinct_cpu.y - previous_distinct_cpu.y;
            human_landed_at_collection_endpoint = human.position()
                == Position {
                    x: last_distinct_cpu.x + delta_x,
                    y: last_distinct_cpu.y + delta_y,
                };
        }
        if plant_tick.is_none()
            && let Some(bomb) = state
                .world()
                .bombs()
                .values()
                .find(|bomb| bomb.owner() == EntityId(2))
        {
            plant_tick = Some(bomb.placed_at());
            plant_cell = Some(bomb.cell());
            plant_reaches_named_target = blast_reaches(bomb.cell(), bomb.range(), named_target);
            finish_at = bomb
                .detonates_at()
                .and_then(|tick| tick.0.checked_add(state.rules().flame_ticks() + 1))
                .map(Tick);
        }
        if finish_at.is_some_and(|tick| state.tick() >= tick) {
            break;
        }
        if position != last_distinct_cpu {
            previous_distinct_cpu = last_distinct_cpu;
            last_distinct_cpu = position;
        }
        prior_human = state.actor(SeatId::One).unwrap().position();
    }
    if native {
        let report = planner.trace_for_test();
        failed_work = report.failed;
        max_work = report.work.into_iter().max().unwrap_or(0);
    }
    let trace = owner.teleporter_trace_for_test();
    let movement_commands: Vec<_> = trace
        .iter()
        .filter(|(kind, _, _, accepted)| *kind == 0 && *accepted)
        .collect();
    let stop_commands: Vec<_> = trace.iter().filter(|(kind, _, _, _)| *kind == 1).collect();
    let state = owner.runtime.state_for_test().unwrap();
    Outcome {
        collection_tick,
        movement_admission_ticks: movement_commands
            .iter()
            .map(|(_, runtime, _, _)| *runtime)
            .collect(),
        movement_sealed_ticks: movement_commands
            .iter()
            .map(|(_, _, sealed, _)| *sealed)
            .collect(),
        movement_update_ticks: movement_updates,
        stop_submission_ticks: stop_commands
            .iter()
            .map(|(_, runtime, _, _)| *runtime)
            .collect(),
        stop_sealed_ticks: stop_commands
            .iter()
            .map(|(_, _, sealed, _)| *sealed)
            .collect(),
        physical_stop_ticks,
        stop_pose_held,
        stop_inputs_cleared,
        stopped_completion_tick,
        cpu_teleport_tick,
        human_teleport_tick,
        cpu_landed_at_human_pose,
        human_landed_at_collection_endpoint,
        plant_tick,
        plant_cell,
        plant_reaches_named_target,
        named_target,
        named_target_existed,
        target_destroyed,
        survived_flames: finish_at.is_some_and(|tick| state.tick() >= tick)
            && state.actor(CPU).unwrap().alive(),
        positions,
        max_work,
        failed_work,
        inputs: owner.runtime.replay_inspection().inputs,
    }
}

pub(super) fn mailbox_is_cleared(inputs: &str) -> bool {
    !inputs.contains("direction: Some")
        && inputs.matches("expires_at: None").count() == 2
        && inputs.matches("inputs: []").count() == 2
        && inputs.matches("manual_actions: []").count() == 2
}

fn blast_reaches(origin: CellId, range: u16, target: CellId) -> bool {
    let aligned = origin.column == target.column || origin.row == target.row;
    let distance = origin.column.abs_diff(target.column) + origin.row.abs_diff(target.row);
    aligned && distance <= range
}

pub(super) fn test_driver(
    profile: CpuProfile,
    difficulty: CpuDifficulty,
    lairs: bool,
    native: bool,
) -> (Driver, CpuPlanner) {
    let settings = RoomSettings::DEFAULT.with_enabled_items(EnabledItems::ALL);
    let configuration = RoomConfiguration::new(
        &[SeatControl::Human, SeatControl::Cpu],
        Some(difficulty),
        BoardSize::FifteenByThirteen,
    )
    .unwrap();
    let mut runtime = RosterRuntime::new_spike_v19_with_navigation(
        RosterIdentity {
            id: RoomId::new("0123456789abcdef0123456789abc647").unwrap(),
            code: RoomCode::new("ABCDEFGH").unwrap(),
            registry_serial: INSTANCE,
            configuration,
        },
        settings,
        StormCloudConfig::Disabled,
        RespawnLairTuning::new(lairs, RespawnLairSettings::DEFAULT),
        SpikePitTuning::new(false, SpikePitSettings::DEFAULT),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(47);
            Ok(())
        }),
        Arc::new(|_| Ok(SimulationSeed::new([47; 32]))),
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
    let scene = controlled_state(basis);
    let notice = runtime.install_running_scene_for_test(scene).unwrap();
    let planner = if native {
        CpuPlanner::new().unwrap()
    } else {
        CpuPlanner::without_workers()
    };
    let controller = CpuController::new(planner.clone(), CPU, INSTANCE, [47; 32], profile);
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
        vec![controller],
    );
    owner.reset_teleporter_trace_for_test();
    owner.publish(notice);
    (owner, planner)
}

fn controlled_state(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        BTreeSet::from([TARGET_BLOCK, HARD_TARGET_BLOCK, HARD_EAST_TARGET_BLOCK]),
        [CellId::new(1, 1), CellId::new(13, 11)],
    )
    .unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(TELEPORTER, PickupKind::Teleporter)]),
    )
    .unwrap();
    let actors = [actor(1, SeatId::One, HUMAN_START), actor(2, CPU, CPU_START)];
    State::new_at_tick_with_assignment_and_seed(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        RoundAssignment::first(),
        SimulationSeed::new([47; 32]),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, cell: CellId) -> ActorInit {
    ActorInit {
        id: EntityId(id),
        seat,
        position: Position::center(cell),
        heading: Direction::North,
        speed: grenadiers_core::game::SpeedLevel::Base,
    }
}
