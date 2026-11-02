use grenadiers_core::{
    cpu::{
        BombKnowledge, Intent, Observation, ObservedActor, WorkBudget, WorldId,
        forecast::ObservedHazardMemory,
        policy::{IntentDebugSummary, StrategicIntent, UtilityMemory},
        survival::{
            TeleporterRecoveryChoice, WORK_ALLOWANCE, plan_corner_refuges_v56_with_origin,
            plan_hard_blocked_births_v43_with_origin, plan_hard_corner_refuges_v57_with_origin,
            plan_hard_farm_yield_v59_with_origin, plan_hard_pit_avoidance_v53_with_origin,
            select_teleporter_recovery_v50,
        },
    },
    game::{CellId, EntityId, PickupKind, Position, SeatId, State, Tick},
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
    pub post_foreign_probe: Option<BasisProbe>,
    pub clear_after_foreign_probe: Option<BasisProbe>,
    pub farm_approach_probe: Option<BasisProbe>,
    pub post_jobs: Vec<PostJobSummary>,
}

#[derive(Debug)]
pub(super) struct PostJobSummary {
    pub tick: Tick,
    pub stop: Tick,
    pub position: Position,
    pub bombs: usize,
    pub flames: usize,
    pub strategy: Option<StrategicIntent>,
    pub had_origin: bool,
    pub work: u32,
    pub intent: Option<Intent>,
}

#[derive(Debug)]
pub(super) struct BasisProbe {
    pub capture: CapturedPlanningBasis,
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
    pub dispatched_work: u32,
    pub planner_intent: Intent,
    pub planner_execution: bool,
}

#[derive(Clone, Debug)]
pub(super) struct CapturedPlanningBasis {
    pub observation: Observation,
    pub hazard_memory: ObservedHazardMemory,
    pub memory: UtilityMemory,
    pub settled_stop_tick: Tick,
    pub had_origin: bool,
}

pub(super) fn run(ticks: usize) -> InteractionOutcome {
    run_for_profile(CpuProfile::HARD_V51, ticks)
}

pub(super) fn run_for_profile(profile: CpuProfile, ticks: usize) -> InteractionOutcome {
    run_inspecting(profile, ticks, |_| {})
}

pub(super) fn run_inspecting(
    profile: CpuProfile,
    ticks: usize,
    mut observe_state: impl FnMut(&State),
) -> InteractionOutcome {
    let (mut owner, planner) =
        scenario_driver_for_cpus(profile, &CPUS, controlled_scene, None, [47; 32]);
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
    let mut post_foreign_probe = None;
    let mut clear_after_foreign_probe = None;
    let mut farm_approach_probe = None;
    let mut post_jobs = Vec::new();
    for _ in 0..ticks {
        let mut inspected = 0;
        let mut captured_index = None;
        let mut post_captured_index = None;
        let mut clear_captured_index = None;
        let mut approach_captured_index = None;
        let mut batch_post_jobs = Vec::new();
        let report = planner.pump_for_test_inspecting(usize::MAX, |basis| {
            let was_captured = basis_probe.is_some();
            inspect_cpu_two_approach_basis(&basis, &mut basis_probe);
            if !was_captured && basis_probe.is_some() {
                captured_index = Some(inspected);
            }
            if basis.observation.parts().recipient == SeatId::Two
                && basis.observation.parts().tick >= Tick(415)
            {
                let ObservedActor::Visible(actor) = &basis.observation.parts().actors[SeatId::Two.index()] else { panic!("CPU must be visible") };
                let BombKnowledge::Visible(bombs) = &basis.observation.parts().bombs else { panic!("bombs must be visible") };
                batch_post_jobs.push((inspected, PostJobSummary {
                    tick: basis.observation.parts().tick,
                    stop: basis.settled_stop_tick,
                    position: actor.position,
                    bombs: bombs.len(),
                    flames: basis.observation.parts().flames.len(),
                    strategy: basis.memory.strategic_intent(),
                    had_origin: basis.escape_origin.is_some(),
                    work: 0,
                    intent: None,
                }));
                if post_foreign_probe.is_none() {
                    post_foreign_probe = Some(probe_basis(&basis));
                    post_captured_index = Some(inspected);
                }
                if clear_after_foreign_probe.is_none()
                    && basis.observation.parts().flames.is_empty()
                    && matches!(&basis.observation.parts().bombs, BombKnowledge::Visible(bombs) if bombs.is_empty())
                {
                    clear_after_foreign_probe = Some(probe_basis(&basis));
                    clear_captured_index = Some(inspected);
                }
                if farm_approach_probe.is_none()
                    && bombs.is_empty()
                    && basis.observation.parts().flames.is_empty()
                    && basis.memory.strategic_intent().and_then(|intent| intent.approach_cell) == actor.position.cell()
                {
                    farm_approach_probe = Some(probe_basis(&basis));
                    approach_captured_index = Some(inspected);
                }
            }
            inspected += 1;
        });
        for (index, probe) in [
            captured_index.zip(basis_probe.as_ref()),
            post_captured_index.zip(post_foreign_probe.as_ref()),
            clear_captured_index.zip(clear_after_foreign_probe.as_ref()),
            approach_captured_index.zip(farm_approach_probe.as_ref()),
        ]
        .into_iter()
        .flatten()
        {
            assert_eq!(report.delivered, report.dequeued);
            assert_eq!(report.work[index], probe.dispatched_work);
            assert_eq!(
                report.planned_intents_by_job[index],
                Some(probe.planner_intent)
            );
        }
        for (index, mut summary) in batch_post_jobs {
            summary.work = report.work[index];
            summary.intent = report.planned_intents_by_job[index];
            post_jobs.push(summary);
        }
        if !report.work.is_empty() {
            pump_batches.push(report.work.clone());
        }
        merge(&mut worker, report);
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        observe_state(state);
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
        post_foreign_probe,
        clear_after_foreign_probe,
        farm_approach_probe,
        post_jobs,
    }
}

fn inspect_cpu_two_approach_basis(basis: &PlanningBasis<'_>, captured: &mut Option<BasisProbe>) {
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
    *captured = Some(probe_basis(basis));
}

fn probe_basis(basis: &PlanningBasis<'_>) -> BasisProbe {
    let ObservedActor::Visible(actor) = &basis.observation.parts().actors[SeatId::Two.index()]
    else {
        panic!("captured CPU must be visible");
    };
    let BombKnowledge::Visible(bombs) = &basis.observation.parts().bombs else {
        panic!("captured bombs must be visible");
    };
    let mut rng = basis.rng.clone();
    let mut work = WorkBudget::new(WORK_ALLOWANCE);
    let plan = match basis.policy {
        CpuPolicy::UtilityV51 => plan_hard_blocked_births_v43_with_origin,
        CpuPolicy::UtilityV55 => plan_hard_pit_avoidance_v53_with_origin,
        CpuPolicy::UtilityV56 => plan_corner_refuges_v56_with_origin,
        CpuPolicy::UtilityV57 => plan_hard_corner_refuges_v57_with_origin,
        CpuPolicy::UtilityV58 => {
            grenadiers_core::cpu::survival::plan_hard_exit_pressure_v58_with_origin
        }
        CpuPolicy::UtilityV59 => plan_hard_farm_yield_v59_with_origin,
        _ => panic!("unsupported interaction characterization profile"),
    };
    let planned = plan(
        basis.observation,
        basis.memory,
        basis.hazard_memory,
        basis.settled_stop_tick,
        basis.escape_origin,
        &mut rng,
        &mut work,
    )
    .unwrap();
    let planner_work = work.spent();
    let TeleporterRecoveryChoice::Ordinary(planned) = select_teleporter_recovery_v50(
        basis.observation,
        basis.hazard_memory,
        basis.settled_stop_tick,
        basis.memory,
        basis.escape_origin,
        planned,
        &mut work,
    ) else {
        panic!("captured scene has no exposed Teleporter recovery target");
    };
    BasisProbe {
        capture: CapturedPlanningBasis {
            observation: basis.observation.clone(),
            hazard_memory: basis.hazard_memory.clone(),
            memory: *basis.memory,
            settled_stop_tick: basis.settled_stop_tick,
            had_origin: basis.escape_origin.is_some(),
        },
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
        planner_work,
        dispatched_work: work.spent(),
        planner_intent: planned.survival.control.intent,
        planner_execution: planned.survival.execution.is_some(),
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
    total
        .planned_intents_by_job
        .extend(next.planned_intents_by_job);
    total.planner_times.extend(next.planner_times);
}
