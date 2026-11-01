use std::collections::BTreeMap;

use grenadiers_core::{
    cpu::{
        Intent, ObservedActor, WorkBudget, WorldId,
        policy::IntentDebugSummary,
        survival::{WORK_ALLOWANCE, plan_hard_blocked_births_v43_with_origin},
    },
    game::{
        ActorInit, Arena, CellId, Direction, EntityId, Position, RoundActorInit, SeatId,
        SimulationSeed, SpikePitPhase, State, Tick, World, fixed_arena,
    },
    room::{CpuProfile, MatchPhase},
};
use tokio::time::Instant;

use super::late_teleporter_owner_support::scenario_driver_for_cpus;
use crate::game::rooms::cpu::{AdmissionTrace, PlanningBasis, PumpReport};

pub(super) const CPU: SeatId = SeatId::Three;
pub(super) const CAPTURE_PIT: CellId = CellId::new(3, 7);
pub(super) const TARGET: CellId = CellId::new(2, 7);
const INSTANCE: u64 = 5_461;

#[derive(Clone, Debug)]
pub(super) struct Sample {
    pub tick: Tick,
    pub position: Position,
    pub alive: bool,
    pub pits: Vec<(CellId, SpikePitPhase, u64)>,
    pub intent: Option<IntentDebugSummary>,
    pub controller: String,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub initial_pits: Vec<CellId>,
    pub samples: Vec<Sample>,
    pub worker: PumpReport,
    pub admission: AdmissionTrace,
    pub result_phase: MatchPhase,
    pub plan_probe: PlanProbe,
}

#[derive(Debug)]
pub(super) struct PlanProbe {
    pub observation_tick: Tick,
    pub settled_stop_tick: Tick,
    pub position: Position,
    pub work: u32,
    pub intent: Intent,
    pub lease_basis_tick: Tick,
    pub lease_window: (Tick, Tick),
    pub lease_hold_ticks: u8,
    pub lease_safe_until: Tick,
}

pub(super) fn run(ticks: usize) -> Outcome {
    let (mut owner, planner) = scenario_driver_for_cpus(
        CpuProfile::HARD_V51,
        &[CPU],
        controlled_scene,
        None,
        [47; 32],
    );
    let world = WorldId {
        room_instance: INSTANCE,
        physical_round: 1,
    };
    let initial_pits = owner
        .runtime
        .state_for_test()
        .unwrap()
        .spike_pits()
        .unwrap()
        .into_iter()
        .map(|pit| pit.cell)
        .collect();
    let mut worker = PumpReport::default();
    let mut samples = Vec::with_capacity(ticks);
    let mut plan_probe = None;
    for _ in 0..ticks {
        let report = planner.pump_for_test_inspecting(usize::MAX, |basis| {
            inspect_first_plan(basis, &mut plan_probe);
        });
        merge(&mut worker, report);
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        let actor = state.actor(CPU).unwrap();
        samples.push(Sample {
            tick: state.tick(),
            position: actor.position(),
            alive: actor.alive(),
            pits: state
                .spike_pits()
                .unwrap()
                .into_iter()
                .map(|pit| (pit.cell, pit.phase, pit.remaining_ticks))
                .collect(),
            intent: owner.cpus[0].intent_debug_summary_for(world),
            controller: owner.cpus[0].controller_state_for_test(),
        });
        if owner.runtime.lifecycle().phase() != MatchPhase::Running {
            break;
        }
    }
    Outcome {
        initial_pits,
        samples,
        worker,
        admission: owner.cpus[0].admission_trace_for_test(),
        result_phase: owner.runtime.lifecycle().phase(),
        plan_probe: plan_probe.expect("the first CPU plan must be inspected"),
    }
}

fn inspect_first_plan(basis: PlanningBasis<'_>, captured: &mut Option<PlanProbe>) {
    if captured.is_some()
        || basis.observation.parts().recipient != CPU
        || basis.observation.parts().tick < Tick(198)
    {
        return;
    }
    let mut rng = basis.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_blocked_births_v43_with_origin(
        basis.observation,
        basis.memory,
        basis.hazard_memory,
        basis.settled_stop_tick,
        basis.escape_origin,
        &mut rng,
        &mut work,
    )
    .unwrap();
    let lease = planned.survival.lease.as_ref().unwrap();
    let ObservedActor::Visible(actor) = &basis.observation.parts().actors[CPU.index()] else {
        panic!("CPU Three must be visible in its own planning basis");
    };
    *captured = Some(PlanProbe {
        observation_tick: basis.observation.parts().tick,
        settled_stop_tick: basis.settled_stop_tick,
        position: actor.position,
        work: work.spent(),
        intent: planned.survival.control.intent,
        lease_basis_tick: lease.basis_tick(),
        lease_window: (lease.start_window().start, lease.start_window().end),
        lease_hold_ticks: lease.hold_ticks(),
        lease_safe_until: lease.safe_until(),
    });
}

fn controlled_scene(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|index| {
        fixed
            .terrain(CellId::new((index % 15) as u16, (index / 15) as u16))
            .unwrap()
    });
    #[rustfmt::skip]
    let blocks = [(3,1),(4,1),(5,1),(7,1),(9,1),(10,1),(11,1),(7,2),
        (1,3),(3,3),(5,3),(6,3),(8,3),(9,3),(11,3),(13,3),(1,4),(5,4),
        (7,4),(9,4),(13,4),(1,5),(2,5),(4,5),(5,5),(6,5),(7,5),(8,5),
        (9,5),(10,5),(12,5),(13,5),(5,6),(9,6),(1,7),(2,7),(4,7),(5,7),
        (6,7),(7,7),(8,7),(9,7),(10,7),(12,7),(13,7),(1,8),(5,8),(7,8),
        (9,8),(13,8),(1,9),(3,9),(5,9),(6,9),(8,9),(9,9),(11,9),(13,9),
        (7,10),(3,11),(4,11),(5,11),(7,11),(9,11),(10,11),(11,11)]
        .into_iter().map(|(column,row)| CellId::new(column,row)).collect();
    let arena = Arena::new(
        terrain,
        blocks,
        [
            CellId::new(1, 1),
            CellId::new(13, 11),
            CellId::new(13, 1),
            CellId::new(1, 11),
        ],
    )
    .unwrap();
    let world =
        World::with_pickups(arena, BTreeMap::new(), BTreeMap::new(), BTreeMap::new()).unwrap();
    // The quiet capture's public blocks and CPU pose are preserved. Empty
    // hidden loot, fresh clocks/controller memory and seed 47 are assumptions.
    State::new_round(
        *basis.rules(),
        world,
        [
            actor(
                1,
                SeatId::One,
                Position::center(CellId::new(1, 1)),
                Direction::East,
            ),
            actor(
                2,
                SeatId::Two,
                Position::center(CellId::new(12, 1)),
                Direction::East,
            ),
            actor(3, CPU, Position { x: 3_584, y: 8_515 }, Direction::North),
            actor(
                4,
                SeatId::Four,
                Position::center(CellId::new(13, 1)),
                Direction::West,
            ),
        ],
        basis.tick(),
        basis.round_assignment(),
        Some(SimulationSeed::new([47; 32])),
    )
    .unwrap()
}

fn actor(id: u32, seat: SeatId, position: Position, heading: Direction) -> RoundActorInit {
    RoundActorInit {
        actor: ActorInit {
            id: EntityId(id),
            seat,
            position,
            heading,
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
