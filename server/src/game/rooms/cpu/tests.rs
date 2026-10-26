use std::sync::{
    Arc,
    atomic::AtomicBool,
    mpsc::{Receiver, sync_channel},
};

use grenadiers_core::{
    cpu::{
        BombKnowledge, CpuRng, ManualIntent, MemoryKey, Observation, ObservationParts,
        ObservedActor, ObservedArena, ObservedRules, UnknownActorFacts, VisibleActor, WorldId,
        policy::UtilityMemory,
    },
    game::{CellId, Direction, EntityId, Position, SeatId, SpeedLevel, Terrain, Tick},
    room::RoomSettings,
};

use super::worker::{CpuPlanner, PlanJob, PlanResult};

fn key() -> MemoryKey {
    MemoryKey {
        world: WorldId {
            room_instance: 7,
            physical_round: 3,
        },
        cpu_seat: SeatId::Two,
    }
}

pub(super) fn observation() -> Observation {
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
        world: key().world,
        tick: Tick(20),
        recipient: SeatId::Two,
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
        actors: [
            ObservedActor::Hidden {
                id: EntityId(1),
                seat: SeatId::One,
                alive: true,
            },
            ObservedActor::Visible(VisibleActor {
                id: EntityId(2),
                seat: SeatId::Two,
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
            }),
        ],
        bombs: BombKnowledge::Visible(Vec::new()),
        flames: Vec::new(),
        oil: Vec::new(),
        pickups: Vec::new(),
    })
    .unwrap()
}

fn job(live: Arc<AtomicBool>) -> (PlanJob, Receiver<PlanResult>) {
    let (sender, receiver) = sync_channel(1);
    (
        PlanJob {
            observation: observation(),
            memory: UtilityMemory::new(key()),
            rng: CpuRng::from_seed(key(), [4; 32]),
            live,
            result: sender,
        },
        receiver,
    )
}

#[test]
fn bounded_worker_returns_a_legal_moving_bomb_plan() {
    let planner = CpuPlanner::new().unwrap();
    let (job, result) = job(Arc::new(AtomicBool::new(true)));
    assert!(planner.try_submit(job));
    let result = result.recv().unwrap();
    assert_eq!(result.plan.intent.manual, Some(ManualIntent::Bomb));
    assert_ne!(
        result.plan.intent.movement,
        grenadiers_core::cpu::MovementIntent::Stop
    );
    assert!((1..=12).contains(&result.plan.hold_ticks));
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
