use std::collections::{BTreeMap, BTreeSet};

use grenadiers_core::{
    cpu::{
        Intent, Observation, ObservedActor, WorkBudget,
        policy::{IntentDebugSummary, StrategicObjectiveKind},
        survival::{
            TeleporterRecoveryChoice, WORK_ALLOWANCE, plan_hard_corner_refuges_v57_with_origin,
            select_teleporter_recovery_v50,
        },
    },
    game::{
        ActorInit, Arena, CellId, Direction, EntityId, PickupKind, Position, RoundActorInit,
        SeatId, SimulationSeed, State, Tick, World, fixed_arena,
    },
    room::CpuProfile,
};
use tokio::time::Instant;

use super::queued_scenario_support::{Environment, driver};
use crate::game::rooms::cpu::{AdmissionTrace, PlanningBasis, PumpReport};

const CPU: SeatId = SeatId::Two;
const INSTANCE: u64 = 5_461;

#[derive(Debug)]
pub(super) struct Basis {
    pub observation: Observation,
    pub stop: Tick,
    pub work: u32,
    pub intent: Intent,
    pub strategy: Option<grenadiers_core::cpu::policy::StrategicIntent>,
    pub execution: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Plant {
    pub id: EntityId,
    pub cell: CellId,
    pub placed_at: Tick,
    pub detonates_at: Tick,
    pub flame_end: Tick,
    pub range: u16,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub basis: Basis,
    pub first_attack: IntentDebugSummary,
    pub plant: Option<Plant>,
    pub rival_death: Option<Tick>,
    pub cpu_death: Option<Tick>,
    pub resolved: bool,
    pub rival_flame_observed: bool,
    pub initial_range: u16,
    pub final_range: u16,
    pub worker: PumpReport,
    pub admissions: AdmissionTrace,
}

pub(super) fn run(profile: CpuProfile, updates: usize) -> Outcome {
    run_with_environment(profile, updates, Environment::Disabled)
}

pub(super) fn run_with_environment(
    profile: CpuProfile,
    updates: usize,
    environment: Environment,
) -> Outcome {
    let (mut owner, planner) = driver(profile, &[CPU], scene, None, [58; 32], environment);
    let initial_range = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .bomb_range();
    let mut basis = None;
    let mut first_attack = None;
    let mut plant = None;
    let mut rival_death = None;
    let mut cpu_death = None;
    let mut resolved = false;
    let mut rival_flame_observed = false;
    let mut worker = PumpReport::default();
    for _ in 0..updates {
        let mut index = 0;
        let mut captured_index = None;
        let report = planner.pump_for_test_inspecting(usize::MAX, |job| {
            if basis.is_none() && ranged(job.observation) {
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
        let world = grenadiers_core::cpu::WorldId {
            room_instance: INSTANCE,
            physical_round: 1,
        };
        if first_attack.is_none()
            && let Some(summary) = owner.cpus[0].intent_debug_summary_for(world)
            && summary.objective == StrategicObjectiveKind::Attack
        {
            first_attack = Some(summary);
        }
        if plant.is_none()
            && let Some((&id, bomb)) = state
                .world()
                .bombs()
                .iter()
                .find(|(_, bomb)| bomb.owner() == cpu.id())
        {
            let detonates_at = bomb.detonates_at().expect("ordinary bomb");
            plant = Some(Plant {
                id,
                cell: bomb.cell(),
                placed_at: bomb.placed_at(),
                detonates_at,
                flame_end: Tick(detonates_at.0 + state.rules().flame_ticks()),
                range: bomb.range(),
            });
        }
        if !state.actor(SeatId::One).unwrap().alive() && rival_death.is_none() {
            rival_death = Some(state.tick());
        }
        if !cpu.alive() && cpu_death.is_none() {
            cpu_death = Some(state.tick());
        }
        if let Some(first) = plant
            && state.tick() == first.detonates_at
        {
            rival_flame_observed =
                state.world().hazards().get(&CellId::new(7, 7)) == Some(&first.flame_end);
        }
        if let Some(first) = plant
            && state.tick().0 == first.flame_end.0 + 1
        {
            resolved = !state.world().bombs().contains_key(&first.id)
                && state.world().hazards().is_empty();
        }
    }
    let final_range = owner
        .runtime
        .state_for_test()
        .unwrap()
        .actor(CPU)
        .unwrap()
        .bomb_range();
    Outcome {
        basis: basis.expect("range pickup must produce an actual queued decision"),
        first_attack: first_attack.expect("Hard must select a visible rival"),
        plant,
        rival_death,
        cpu_death,
        resolved,
        rival_flame_observed,
        initial_range,
        final_range,
        worker,
        admissions: owner.cpus[0].admission_trace_for_test(),
    }
}

fn ranged(observation: &Observation) -> bool {
    matches!(&observation.parts().actors[CPU.index()], ObservedActor::Visible(cpu) if cpu.bomb_range == 3)
}

fn inspect(job: &PlanningBasis<'_>) -> Basis {
    assert_eq!(*job.policy, grenadiers_core::room::CpuPolicy::UtilityV57);
    assert!(job.escape_origin.is_none());
    let mut rng = job.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let planned = plan_hard_corner_refuges_v57_with_origin(
        job.observation,
        job.memory,
        job.hazard_memory,
        job.settled_stop_tick,
        None,
        &mut rng,
        &mut work,
    )
    .unwrap();
    let TeleporterRecoveryChoice::Ordinary(planned) = select_teleporter_recovery_v50(
        job.observation,
        job.hazard_memory,
        job.settled_stop_tick,
        job.memory,
        None,
        planned,
        &mut work,
    ) else {
        panic!("fixture has no Teleporter target")
    };
    Basis {
        observation: job.observation.clone(),
        stop: job.settled_stop_tick,
        work: work.spent(),
        intent: planned.survival.control.intent,
        strategy: planned.survival.control.next_memory.strategic_intent(),
        execution: planned.survival.execution.is_some(),
    }
}

fn scene(basis: &State) -> State {
    let fixed = fixed_arena();
    let terrain = std::array::from_fn(|i| {
        fixed
            .terrain(CellId::new((i % 15) as u16, (i / 15) as u16))
            .unwrap()
    });
    let arena = Arena::new(
        terrain,
        BTreeSet::from([CellId::new(7, 8)]),
        [
            CellId::new(1, 1),
            CellId::new(13, 1),
            CellId::new(1, 11),
            CellId::new(13, 11),
        ],
    )
    .unwrap();
    let world = World::with_pickups(
        arena,
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::from([(CellId::new(5, 5), PickupKind::BlastRange)]),
    )
    .unwrap();
    let actors = [
        (SeatId::One, CellId::new(7, 7)),
        (CPU, CellId::new(5, 5)),
        (SeatId::Three, CellId::new(1, 11)),
        (SeatId::Four, CellId::new(13, 11)),
    ]
    .map(|(seat, cell)| RoundActorInit {
        actor: ActorInit {
            id: EntityId(seat.index() as u32 + 1),
            seat,
            position: Position::center(cell),
            heading: Direction::East,
            speed: basis.rules().room_settings().unwrap().starting_speed(),
        },
        participating: true,
    });
    State::new_round(
        *basis.rules(),
        world,
        actors,
        basis.tick(),
        basis.round_assignment(),
        Some(SimulationSeed::new([58; 32])),
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
