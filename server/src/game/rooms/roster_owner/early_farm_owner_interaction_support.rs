use grenadiers_core::{
    cpu::{
        BombKnowledge, Intent, ObservedActor, WorkBudget, WorldId,
        policy::IntentDebugSummary,
        survival::{WORK_ALLOWANCE, plan_hard_blocked_births_v43_with_origin},
    },
    game::{CellId, EntityId, PickupKind, Position, SeatId, Tick},
    room::{CpuPolicy, CpuProfile},
};
use tokio::time::Instant;

use super::{
    early_farm_owner_support::controlled_scene,
    late_teleporter_owner_support::scenario_driver_for_cpus,
};
use crate::game::rooms::cpu::{AdmissionTrace, PlanningBasis, PumpReport};

const INSTANCE: u64 = 5_461;
pub(super) const CPUS: [SeatId; 3] = [SeatId::Two, SeatId::Three, SeatId::Four];

#[derive(Clone, Debug)]
pub(super) struct CpuTwoSample {
    pub tick: Tick,
    pub position: Position,
    pub intent: Option<IntentDebugSummary>,
    pub controller: String,
}

#[derive(Debug)]
pub(super) struct InteractionOutcome {
    pub initial_positions: [Position; 3],
    pub first_plants: [Option<Tick>; 3],
    pub cpu_two_samples: Vec<CpuTwoSample>,
    pub pump_batches: Vec<Vec<u32>>,
    pub worker: PumpReport,
    pub admissions: [AdmissionTrace; 3],
    pub basis_probe: BasisProbe,
}

#[derive(Debug)]
pub(super) struct BasisProbe {
    pub policy: CpuPolicy,
    pub tick: Tick,
    pub settled_stop_tick: Tick,
    pub position: Position,
    pub teleporter_enabled: bool,
    pub visible_bombs: usize,
    pub visible_flames: usize,
    pub blocks: usize,
    pub running_updates: u16,
    pub exposed_teleporter: bool,
    pub had_origin: bool,
    pub planner_work: u32,
    pub planner_intent: Intent,
    pub planner_execution: bool,
}

pub(super) fn run(ticks: usize) -> InteractionOutcome {
    let (mut owner, planner) = scenario_driver_for_cpus(
        CpuProfile::HARD_V51,
        &CPUS,
        controlled_scene,
        None,
        [47; 32],
    );
    let state = owner.runtime.state_for_test().unwrap();
    let initial_positions = CPUS.map(|seat| state.actor(seat).unwrap().position());
    let world = WorldId {
        room_instance: INSTANCE,
        physical_round: 1,
    };
    let mut first_plants = [None; 3];
    let mut cpu_two_samples = Vec::with_capacity(ticks);
    let mut pump_batches = Vec::new();
    let mut worker = PumpReport::default();
    let mut basis_probe = None;
    for _ in 0..ticks {
        let report = planner.pump_for_test_inspecting(usize::MAX, |basis| {
            inspect_cpu_two_approach_basis(basis, &mut basis_probe);
        });
        if !report.work.is_empty() {
            pump_batches.push(report.work.clone());
        }
        merge(&mut worker, report);
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        for (index, seat) in CPUS.into_iter().enumerate() {
            first_plants[index] = first_plants[index].or_else(|| {
                state
                    .world()
                    .bombs()
                    .values()
                    .filter(|bomb| bomb.owner() == EntityId(index as u32 + 2))
                    .map(|bomb| bomb.placed_at())
                    .min()
            });
            debug_assert_eq!(owner.cpus[index].seat(), seat);
        }
        cpu_two_samples.push(CpuTwoSample {
            tick: state.tick(),
            position: state.actor(SeatId::Two).unwrap().position(),
            intent: owner.cpus[0].intent_debug_summary_for(world),
            controller: owner.cpus[0].controller_state_for_test(),
        });
    }
    InteractionOutcome {
        initial_positions,
        first_plants,
        cpu_two_samples,
        pump_batches,
        worker,
        admissions: std::array::from_fn(|index| owner.cpus[index].admission_trace_for_test()),
        basis_probe: basis_probe.expect("CPU Two must queue a post-bomb approach basis"),
    }
}

fn inspect_cpu_two_approach_basis(basis: PlanningBasis<'_>, captured: &mut Option<BasisProbe>) {
    if captured.is_some() || basis.observation.parts().recipient != SeatId::Two {
        return;
    }
    let ObservedActor::Visible(actor) = &basis.observation.parts().actors[SeatId::Two.index()]
    else {
        return;
    };
    let BombKnowledge::Visible(bombs) = &basis.observation.parts().bombs else {
        return;
    };
    if actor.position.cell() != Some(CellId::new(11, 1))
        || actor.position.x > Position::center(CellId::new(11, 1)).x
        || bombs.is_empty()
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
    *captured = Some(BasisProbe {
        policy: *basis.policy,
        tick: basis.observation.parts().tick,
        settled_stop_tick: basis.settled_stop_tick,
        position: actor.position,
        teleporter_enabled: basis
            .observation
            .parts()
            .rules
            .settings
            .enabled_items()
            .contains_teleporter(),
        visible_bombs: bombs.len(),
        visible_flames: basis.observation.parts().flames.len(),
        blocks: basis.observation.parts().arena.blocks.len(),
        running_updates: basis.observation.parts().rules.running_updates,
        exposed_teleporter: basis
            .observation
            .parts()
            .pickups
            .iter()
            .any(|pickup| pickup.kind == PickupKind::Teleporter),
        had_origin: basis.escape_origin.is_some(),
        planner_work: work.spent(),
        planner_intent: planned.survival.control.intent,
        planner_execution: planned.survival.execution.is_some(),
    });
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
