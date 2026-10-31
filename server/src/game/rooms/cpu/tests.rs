use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{Receiver, sync_channel},
};

use grenadiers_core::{
    cpu::{
        BombKnowledge, CpuRng, ManualIntent, MemoryKey, Observation, ObservationParts,
        ObservedActor, ObservedArena, ObservedPickup, ObservedRules, UnknownActorFacts,
        VisibleActor, WorldId, forecast::ObservedHazardMemory, policy::UtilityMemory,
    },
    game::{CellId, Direction, EntityId, PickupKind, Position, SeatId, SpeedLevel, Terrain, Tick},
    room::RoomSettings,
};

use super::worker::{CpuPlanner, PlanJob, PlanResult};

fn key() -> MemoryKey {
    key_for(SeatId::Two)
}

fn key_for(cpu_seat: SeatId) -> MemoryKey {
    MemoryKey {
        world: WorldId {
            room_instance: 7,
            physical_round: 3,
        },
        cpu_seat,
    }
}

pub(super) fn observation() -> Observation {
    observation_for(SeatId::Two)
}

pub(super) fn observation_for(cpu_seat: SeatId) -> Observation {
    use grenadiers_core::game::arena::{ARENA_CELL_COUNT, ARENA_HEIGHT, ARENA_WIDTH};
    let mut terrain = vec![Terrain::Floor; ARENA_CELL_COUNT];
    for column in 0..ARENA_WIDTH {
        terrain[usize::from(column)] = Terrain::Wall;
        terrain[usize::from(ARENA_HEIGHT - 1) * usize::from(ARENA_WIDTH) + usize::from(column)] =
            Terrain::Wall;
    }
    for row in 0..ARENA_HEIGHT {
        terrain[usize::from(row) * usize::from(ARENA_WIDTH)] = Terrain::Wall;
        terrain[usize::from(row) * usize::from(ARENA_WIDTH) + usize::from(ARENA_WIDTH - 1)] =
            Terrain::Wall;
    }
    Observation::new(ObservationParts {
        world: key_for(cpu_seat).world,
        tick: Tick(20),
        recipient: cpu_seat,
        rules: ObservedRules {
            version: 5,
            tick_rate: 60,
            running_updates: 20,
            remaining_ticks: 1_000,
            settings: RoomSettings::DEFAULT,
        },
        arena: ObservedArena {
            width: ARENA_WIDTH,
            height: ARENA_HEIGHT,
            terrain,
            blocks: vec![CellId::new(8, 7)],
        },
        actors: SeatId::DUEL
            .map(|seat| {
                if seat == cpu_seat {
                    ObservedActor::Visible(VisibleActor {
                        id: EntityId(seat.index() as u32 + 1),
                        seat,
                        position: Position::center(CellId::new(7, 7)),
                        heading: Direction::West,
                        speed: SpeedLevel::Base,
                        alive: true,
                        bomb_capacity: 1,
                        bomb_range: 2,
                        live_bomb_count: 0,
                        equipment: None,
                        weapon: None,
                        effects: Vec::new(),
                        damage_grace_remaining_ticks: 0,
                        voluntary_actions_frozen: false,
                        last_teleport_tick: None,
                        unknown: UnknownActorFacts::ALL,
                    })
                } else {
                    ObservedActor::Hidden {
                        id: EntityId(seat.index() as u32 + 1),
                        seat,
                        alive: true,
                    }
                }
            })
            .into(),
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    })
    .unwrap()
}

fn job(live: Arc<AtomicBool>) -> (PlanJob, Receiver<PlanResult>) {
    let (sender, receiver) = sync_channel(1);
    let observation = observation();
    let mut hazard_memory = ObservedHazardMemory::new(key());
    hazard_memory.observe(&observation).unwrap();
    (
        PlanJob {
            policy: grenadiers_core::room::CpuPolicy::UtilityV5,
            settled_stop_tick: observation.parts().tick,
            observation,
            memory: UtilityMemory::new(key()),
            hazard_memory,
            rng: CpuRng::from_seed(key(), [4; 32]),
            escape_origin: None,
            live,
            result: sender,
        },
        receiver,
    )
}

fn respawn_job(live: Arc<AtomicBool>) -> (PlanJob, Receiver<PlanResult>) {
    let observation = respawn_observation();
    let mut hazards = ObservedHazardMemory::new(key());
    hazards.observe(&observation).unwrap();
    let (result, receiver) = sync_channel(1);
    (
        PlanJob {
            policy: grenadiers_core::room::CpuPolicy::UtilityV22,
            settled_stop_tick: observation.parts().tick,
            observation,
            memory: UtilityMemory::new(key()),
            hazard_memory: hazards,
            rng: CpuRng::from_seed(key(), [21; 32]),
            escape_origin: None,
            live,
            result,
        },
        receiver,
    )
}

pub(super) fn respawn_observation() -> Observation {
    let base = observation();
    let mut parts = base.into_parts();
    parts.rules.version = 9;
    let timers = parts
        .actors
        .iter()
        .map(|actor| match actor {
            ObservedActor::Visible(_) => Some(0),
            ObservedActor::Hidden { .. } => None,
        })
        .collect();
    Observation::new_rules_v9(
        parts,
        grenadiers_core::cpu::ObservedEnvironment::new(
            grenadiers_core::game::StormCloudConfig::Disabled,
            None,
            timers,
        ),
        None,
    )
    .unwrap()
}

#[test]
fn bounded_worker_dispatches_rules_nine_only_through_the_respawn_planner() {
    let planner = CpuPlanner::without_workers();
    let (job, _receiver) = respawn_job(Arc::new(AtomicBool::new(true)));
    assert!(planner.try_submit(job));
    let report = planner.pump_for_test(1);
    assert_eq!(report.delivered, 1);
    assert_eq!(report.failed, 0);
    assert!(report.work[0] <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
}

#[test]
fn bounded_worker_returns_a_legal_moving_bomb_plan() {
    let planner = CpuPlanner::new().unwrap();
    let (job, result) = job(Arc::new(AtomicBool::new(true)));
    assert!(planner.try_submit(job));
    let result = result.recv().unwrap();
    assert_eq!(result.plan.control.intent.manual, Some(ManualIntent::Bomb));
    assert_ne!(
        result.plan.control.intent.movement,
        grenadiers_core::cpu::MovementIntent::Stop
    );
    assert!((1..=12).contains(&result.plan.control.hold_ticks));
}

#[test]
fn planner_queue_rejects_the_thirty_third_outstanding_job() {
    let planner = CpuPlanner::without_workers();
    let mut results = Vec::new();
    for _ in 0..32 {
        let (job, result) = job(Arc::new(AtomicBool::new(true)));
        assert!(planner.try_submit(job));
        results.push(result);
    }
    let (overflow, _) = job(Arc::new(AtomicBool::new(true)));
    assert!(!planner.try_submit(overflow));
    assert_eq!(results.len(), 32);
}

#[test]
fn canceled_job_cannot_publish_a_result() {
    let planner = CpuPlanner::new().unwrap();
    let live = Arc::new(AtomicBool::new(false));
    let (job, result) = job(live);
    assert!(planner.try_submit(job));
    drop(planner);
    assert!(result.recv().is_err());
}

#[test]
fn zero_worker_pump_is_bounded_and_limit_zero_preserves_the_queue() {
    let planner = CpuPlanner::without_workers();
    let mut results = Vec::new();
    for _ in 0..32 {
        let (job, result) = job(Arc::new(AtomicBool::new(true)));
        assert!(planner.try_submit(job));
        results.push(result);
    }
    assert_eq!(planner.pump_for_test(0).dequeued, 0);
    let report = planner.pump_for_test(usize::MAX);
    assert_eq!(report.dequeued, 32);
    assert_eq!(report.delivered, 32);
    assert!(report.work.iter().all(|spent| *spent <= 65_536));
    assert_eq!(report.planner_times.len(), 32);
    assert!(results.into_iter().all(|result| result.recv().is_ok()));
}

#[test]
#[should_panic(expected = "only a zero-worker planner may be pumped")]
fn pump_rejects_a_real_worker_pool() {
    CpuPlanner::new().unwrap().pump_for_test(0);
}

#[test]
fn pump_separates_cancel_failure_and_delivery_drop_without_blocking() {
    let planner = CpuPlanner::without_workers();

    let (cancelled, cancelled_result) = job(Arc::new(AtomicBool::new(false)));
    assert!(planner.try_submit(cancelled));
    let report = planner.pump_for_test(1);
    assert_eq!((report.cancelled, report.work.as_slice()), (1, &[0][..]));
    assert!(report.planner_times.is_empty());
    assert!(cancelled_result.recv().is_err());

    let (mut failed, failed_result) = job(Arc::new(AtomicBool::new(true)));
    let mut wrong = key();
    wrong.world.physical_round += 1;
    failed.memory = UtilityMemory::new(wrong);
    assert!(planner.try_submit(failed));
    let report = planner.pump_for_test(1);
    assert_eq!(report.failed, 1);
    assert_eq!(report.planner_times.len(), 1);
    assert!(failed_result.recv().is_err());

    let (disconnected, disconnected_receiver) = job(Arc::new(AtomicBool::new(true)));
    drop(disconnected_receiver);
    assert!(planner.try_submit(disconnected));
    let report = planner.pump_for_test(1);
    assert_eq!(report.dropped, 1);

    let (mut dropped, original_receiver) = job(Arc::new(AtomicBool::new(true)));
    drop(original_receiver);
    let (full_sender, full_receiver) = sync_channel(0);
    dropped.result = full_sender;
    assert!(planner.try_submit(dropped));
    let report = planner.pump_for_test(1);
    assert_eq!(report.dropped, 1);
    assert_eq!(report.planner_times.len(), 1);
    drop(full_receiver);
}

#[test]
fn delivered_job_carries_the_actual_advanced_rng() {
    let planner = CpuPlanner::without_workers();
    let (mut submitted, result) = job(Arc::new(AtomicBool::new(true)));
    let mut parts = submitted.observation.into_parts();
    parts.arena.blocks.clear();
    parts.pickups = vec![
        ObservedPickup {
            cell: CellId::new(8, 7),
            kind: PickupKind::BombCapacity,
        },
        ObservedPickup {
            cell: CellId::new(7, 6),
            kind: PickupKind::BombCapacity,
        },
    ];
    submitted.observation = Observation::new(parts).unwrap();
    submitted.hazard_memory = ObservedHazardMemory::new(key());
    submitted
        .hazard_memory
        .observe(&submitted.observation)
        .unwrap();
    let submitted_draws = submitted.rng.draws();
    let mut expected_rng = submitted.rng.clone();
    let mut expected_work =
        grenadiers_core::cpu::WorkBudget::new(grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    let expected = grenadiers_core::cpu::survival::plan(
        &submitted.observation,
        &submitted.memory,
        &submitted.hazard_memory,
        submitted.settled_stop_tick,
        &mut expected_rng,
        &mut expected_work,
    )
    .unwrap();
    assert!(expected_rng.draws() > submitted_draws);
    assert!(planner.try_submit(submitted));
    let report = planner.pump_for_test(1);
    assert_eq!(report.delivered, 1);
    assert_eq!(report.work, vec![expected_work.spent()]);
    let mut actual = result.recv().unwrap();
    assert_eq!(actual.plan, expected);
    assert_eq!(actual.rng.key(), expected_rng.key());
    assert_eq!(actual.rng.draws(), expected_rng.draws());
    assert_eq!(
        actual.rng.bounded_u32(1_000).unwrap(),
        expected_rng.bounded_u32(1_000).unwrap()
    );
}
