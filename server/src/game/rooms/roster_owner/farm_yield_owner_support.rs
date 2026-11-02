use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{BombKnowledge, Observation, ObservedActor, policy::StrategicIntent},
    game::{
        ActorInit, Arena, CellId, Direction, EnabledItems, EntityId, PickupKind, Position, SeatId,
        SimulationSeed, State, Tick, World, fixed_arena,
    },
    room::CpuProfile,
};
use tokio::time::Instant;

use super::{
    queued_plan_probe_support::inspect,
    queued_scenario_support::{Environment, duel_driver},
};
use crate::game::rooms::cpu::{AdmissionTrace, PumpReport};

const CPU: SeatId = SeatId::Two;
const START: CellId = CellId::new(5, 5);
const FARM_TARGET: CellId = CellId::new(3, 7);
const SECOND_BLOCK: CellId = CellId::new(7, 7);

#[derive(Clone, Copy, Debug)]
pub(super) struct Plant {
    pub cell: CellId,
    pub placed_at: Tick,
    pub detonates_at: Tick,
    pub flame_end: Tick,
    pub range: u16,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub basis_tick: Tick,
    pub settled_stop_tick: Tick,
    pub work: u32,
    pub intent: grenadiers_core::cpu::Intent,
    pub strategy: StrategicIntent,
    pub execution: bool,
    pub initial_range: u16,
    pub final_range: u16,
    pub plant: Plant,
    pub destroyed_blocks: usize,
    pub target_destroyed: bool,
    pub second_block_destroyed: bool,
    pub open_flame_observed: bool,
    pub flame_horizon_observed: bool,
    pub resolved: bool,
    pub cpu_death: Option<Tick>,
    pub worker: PumpReport,
    pub admissions: AdmissionTrace,
}

pub(super) fn run(profile: CpuProfile, updates: usize) -> Outcome {
    let (mut owner, planner) = duel_driver(profile, scene, [58; 32], Environment::Disabled);
    let initial_range = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .bomb_range();
    let mut basis = None;
    let mut plant = None;
    let mut destroyed_blocks = 0;
    let mut target_destroyed = false;
    let mut second_block_destroyed = false;
    let mut open_flame_observed = false;
    let mut flame_horizon_observed = false;
    let mut resolved = false;
    let mut cpu_death = None;
    let mut worker = PumpReport::default();
    for _ in 0..updates {
        let mut captured_index = None;
        let mut index = 0;
        let report = planner.pump_for_test_inspecting(usize::MAX, |job| {
            if basis.is_none() && is_collected_farm_basis(job.observation) {
                assert!(job.escape_origin.is_none());
                basis = Some(inspect(&job));
                captured_index = Some(index);
            }
            index += 1;
        });
        if let Some(index) = captured_index {
            let captured = basis.as_ref().unwrap();
            assert_eq!(report.work[index], captured.work);
            assert_eq!(report.planned_intents_by_job[index], Some(captured.intent));
        }
        merge(&mut worker, report);
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        let cpu = state.actor(CPU).unwrap();
        if plant.is_none()
            && let Some(bomb) = state
                .world()
                .bombs()
                .values()
                .find(|bomb| bomb.owner() == cpu.id())
        {
            let detonates_at = bomb.detonates_at().expect("ordinary timed bomb");
            plant = Some(Plant {
                cell: bomb.cell(),
                placed_at: bomb.placed_at(),
                detonates_at,
                flame_end: Tick(detonates_at.0 + state.rules().flame_ticks()),
                range: bomb.range(),
            });
        }
        if !cpu.alive() && cpu_death.is_none() {
            cpu_death = Some(state.tick());
        }
        if let Some(first) = plant
            && state.tick() == first.detonates_at
        {
            destroyed_blocks = 6 - state.world().arena().blocks().len();
            target_destroyed = !state.world().arena().blocks().contains(&FARM_TARGET);
            second_block_destroyed = !state.world().arena().blocks().contains(&SECOND_BLOCK);
            open_flame_observed =
                state.world().hazards().get(&CellId::new(3, 6)) == Some(&first.flame_end);
            flame_horizon_observed = !state.world().hazards().is_empty()
                && state
                    .world()
                    .hazards()
                    .values()
                    .all(|expiry| *expiry == first.flame_end);
        }
        if let Some(first) = plant
            && state.tick().0 == first.flame_end.0 + 1
        {
            resolved = state.world().bombs().is_empty()
                && state.world().hazards().is_empty()
                && state.actor(CPU).unwrap().alive();
            break;
        }
    }
    let final_range = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .bomb_range();
    let basis = basis.expect("natural range pickup must queue a farming decision");
    Outcome {
        basis_tick: basis.observation.parts().tick,
        settled_stop_tick: basis.stop,
        work: basis.work,
        intent: basis.intent,
        strategy: basis.strategy.expect("farming basis must retain strategy"),
        execution: basis.execution,
        initial_range,
        final_range,
        plant: plant.expect("baseline farming approach must plant"),
        destroyed_blocks,
        target_destroyed,
        second_block_destroyed,
        open_flame_observed,
        flame_horizon_observed,
        resolved,
        cpu_death,
        worker,
        admissions: owner.cpus[0].admission_trace_for_test(),
    }
}

fn is_collected_farm_basis(observation: &Observation) -> bool {
    let ObservedActor::Visible(cpu) = &observation.parts().actors[CPU.index()] else {
        return false;
    };
    let blocks = &observation.parts().arena.blocks;
    cpu.position == Position::center(START)
        && cpu.bomb_range == 3
        && blocks.len() == 6
        && [
            FARM_TARGET,
            SECOND_BLOCK,
            CellId::new(10, 9),
            CellId::new(12, 9),
            CellId::new(11, 8),
            CellId::new(11, 10),
        ]
        .iter()
        .all(|cell| blocks.contains(cell))
        && observation.parts().pickups.is_empty()
        && matches!(&observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
        && observation.parts().flames.is_empty()
        && observation.parts().oil.is_empty()
        && observation.parts().rules.settings.enabled_items() == EnabledItems::ALL
        && observation.environment().unwrap().storm_cloud().is_none()
        && observation.respawn_lairs().is_none()
        && observation.spike_pits().is_none()
}

fn scene(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    let blocks = BTreeSet::from([
        FARM_TARGET,
        SECOND_BLOCK,
        CellId::new(10, 9),
        CellId::new(12, 9),
        CellId::new(11, 8),
        CellId::new(11, 10),
    ]);
    let arena = Arena::new(terrain, blocks, [CellId::new(1, 1), CellId::new(13, 11)]).unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(START, PickupKind::BlastRange)]),
    )
    .unwrap();
    let actors = [
        (SeatId::One, CellId::new(11, 9), Direction::West),
        (CPU, START, Direction::East),
    ]
    .map(|(seat, cell, heading)| ActorInit {
        id: EntityId(seat.index() as u32 + 1),
        seat,
        position: Position::center(cell),
        heading,
        speed: basis.rules().room_settings().unwrap().starting_speed(),
    });
    State::new_at_tick_with_assignment_and_seed(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        basis.round_assignment(),
        SimulationSeed::new([58; 32]),
    )
    .unwrap()
}

fn merge(total: &mut PumpReport, next: PumpReport) {
    total.dequeued += next.dequeued;
    total.delivered += next.delivered;
    total.cancelled += next.cancelled;
    total.failed += next.failed;
    total.dropped += next.dropped;
    total.work.extend(next.work);
    total.planned_intents.extend(next.planned_intents);
    total
        .planned_intents_by_job
        .extend(next.planned_intents_by_job);
    total.planner_times.extend(next.planner_times);
}
