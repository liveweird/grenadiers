use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{ManualIntent, MovementIntent, WorldId, policy::IntentDebugSummary},
    game::{
        ActorInit, Arena, CellId, Direction, EntityId, Position, RoundActorInit, SeatId,
        SimulationSeed, State, Tick, World, fixed_arena,
    },
    room::CpuProfile,
};
use tokio::time::Instant;

use super::late_teleporter_owner_support::scenario_driver_with_cpu_seed;
use crate::game::rooms::cpu::{AdmissionTrace, PumpReport};

pub(super) const CPU: SeatId = SeatId::Two;
const INSTANCE: u64 = 5_461;

#[derive(Clone, Debug)]
pub(super) struct Sample {
    pub tick: Tick,
    pub position: Position,
    pub intent: Option<IntentDebugSummary>,
    pub controller: String,
    pub bombs: usize,
    pub blocks: usize,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub initial_blocks: usize,
    pub initial_position: Position,
    pub samples: Vec<Sample>,
    pub worker: PumpReport,
    pub admission: AdmissionTrace,
    pub first_bomb: Option<Tick>,
    pub cloud: grenadiers_core::game::StormCloudView,
    pub lairs: Vec<CellId>,
    pub pits: Vec<CellId>,
}

pub(super) fn run(seed: u8, ticks: usize) -> Outcome {
    let (mut owner, planner) = scenario_driver_with_cpu_seed(
        CpuProfile::HARD_V51,
        CPU,
        controlled_scene,
        None,
        [seed; 32],
    );
    let world = WorldId {
        room_instance: INSTANCE,
        physical_round: 1,
    };
    let state = owner.runtime.state_for_test().unwrap();
    let initial_position = state.actor(CPU).unwrap().position();
    let initial_blocks = state.world().arena().blocks().len();
    let cloud = state.storm_cloud().unwrap();
    let lairs = state
        .lair_statuses()
        .unwrap()
        .into_iter()
        .map(|lair| lair.cell)
        .collect();
    let pits = state
        .spike_pits()
        .unwrap()
        .into_iter()
        .map(|pit| pit.cell)
        .collect();
    let mut samples = Vec::with_capacity(ticks);
    let mut worker = PumpReport::default();
    let mut first_bomb = None;
    for _ in 0..ticks {
        merge(&mut worker, planner.pump_for_test(usize::MAX));
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        let owned_bombs = state
            .world()
            .bombs()
            .values()
            .filter(|bomb| bomb.owner() == EntityId(2))
            .collect::<Vec<_>>();
        first_bomb = first_bomb.or_else(|| owned_bombs.first().map(|bomb| bomb.placed_at()));
        samples.push(Sample {
            tick: state.tick(),
            position: state.actor(CPU).unwrap().position(),
            intent: owner.cpus[0].intent_debug_summary_for(world),
            controller: owner.cpus[0].controller_state_for_test(),
            bombs: owned_bombs.len(),
            blocks: state.world().arena().blocks().len(),
        });
    }
    Outcome {
        initial_blocks,
        initial_position,
        samples,
        worker,
        admission: owner.cpus[0].admission_trace_for_test(),
        first_bomb,
        cloud,
        lairs,
        pits,
    }
}

pub(super) fn controlled_scene(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    #[rustfmt::skip]
    let blocks = [(5,1),(9,1),(3,2),(7,2),(11,2),(3,3),(4,3),(10,3),(11,3),
        (1,4),(3,4),(7,4),(11,4),(13,4),(1,5),(2,5),(3,5),(4,5),(6,5),
        (8,5),(10,5),(11,5),(12,5),(13,5),(1,7),(2,7),(3,7),(4,7),(6,7),
        (8,7),(10,7),(11,7),(12,7),(13,7),(1,8),(3,8),(7,8),(11,8),(13,8),
        (3,9),(4,9),(10,9),(11,9),(3,10),(7,10),(11,10),(5,11),(9,11)]
        .into_iter().map(|(column,row)| CellId::new(column,row)).collect::<BTreeSet<_>>();
    let arena = Arena::new(
        terrain,
        blocks,
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap();
    // The capture exposes no loot placement. Empty concealed, pending and
    // exposed layers are a controlled fixture assumption, not recovered history.
    let world =
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), BTreeMap::new()).unwrap();
    let actors = [
        actor(1, SeatId::One, 1_536, 1_536),
        actor(2, CPU, 13_824, 1_536),
        actor(3, SeatId::Three, 1_536, 11_776),
        actor(4, SeatId::Four, 13_824, 11_776),
    ];
    // Public spawn geometry is preserved. The fresh clock, seed and all
    // residual controller/environment state are controlled assumptions.
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

fn actor(id: u32, seat: SeatId, x: i32, y: i32) -> RoundActorInit {
    RoundActorInit {
        actor: ActorInit {
            id: EntityId(id),
            seat,
            position: Position { x, y },
            heading: Direction::East,
            speed: grenadiers_core::game::SpeedLevel::Base,
        },
        participating: true,
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

pub(super) fn plan_counts(outcome: &Outcome) -> (usize, usize, usize) {
    let moving = outcome
        .worker
        .planned_intents
        .iter()
        .filter(|intent| matches!(intent.movement, MovementIntent::Direction(_)))
        .count();
    let stopped = outcome
        .worker
        .planned_intents
        .iter()
        .filter(|intent| intent.movement == MovementIntent::Stop)
        .count();
    let planting = outcome
        .worker
        .planned_intents
        .iter()
        .filter(|intent| intent.manual == Some(ManualIntent::Bomb))
        .count();
    (moving, stopped, planting)
}
