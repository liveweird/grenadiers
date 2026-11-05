use std::{fmt::Write, sync::Arc};

use grenadiers_core::{
    cpu::{Intent, Observation, WorldId, policy::StrategicIntent, survival::ExecutionCertificate},
    game::{SeatId, State, StepEventKind, Tick},
    room::{CpuDifficulty, CpuPolicy},
};
use tokio::{sync::watch, time::Instant};

#[path = "hard_approach_diagnostic_support.rs"]
mod support;

use super::{
    RosterOwnerNotice, driver::Driver, live_cpu_evaluation::configuration,
    live_cpu_evaluation_types::CPU_SEATS,
};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{AdmissionTrace, CpuController, CpuPlanner, PlannedResultWitness},
        replay_v37::RecordingRosterRuntime,
        roster_runtime::{EvaluationOperation, RosterAction},
    },
    round_seed::RoundSeedSource,
};

const ARENA_SEED: u8 = 53;
const CPU_SEED_SET: u8 = 71;
const SIMULATION_ROOT_SEED: u8 = 113;
const TICKS: usize = 720;
const PRIORITY: SeatId = SeatId::Four;

#[derive(Debug)]
struct JobTrace {
    run_tick: u64,
    basis_tick: Tick,
    policy: CpuPolicy,
    recipient: SeatId,
    actors: Vec<(SeatId, bool, Option<grenadiers_core::game::CellId>)>,
    memory: String,
    history: String,
    incoming: Option<StrategicIntent>,
    settled_stop_tick: Tick,
    rng_key: grenadiers_core::cpu::MemoryKey,
    rng_before: u64,
    rng_after: u64,
    output: Option<Intent>,
    outgoing: Option<StrategicIntent>,
    plan: String,
    certificate: Option<&'static str>,
    contact: bool,
    work: u32,
}

#[derive(Debug)]
struct ActionTrace {
    run_tick: u64,
    seat: SeatId,
    action: String,
}

#[derive(Debug)]
struct StrategicTrace {
    run_tick: u64,
    cell: Option<grenadiers_core::game::CellId>,
    intent: grenadiers_core::cpu::policy::IntentDebugSummary,
}

#[derive(Debug)]
struct EventTrace {
    run_tick: u64,
    event: String,
}

#[derive(Debug)]
struct DiagnosticOutcome {
    initial: State,
    first_bases: [Option<Observation>; 3],
    jobs: Vec<JobTrace>,
    actions: Vec<ActionTrace>,
    strategic: Vec<StrategicTrace>,
    events: Vec<EventTrace>,
    admissions: Vec<(u64, SeatId, AdmissionTrace)>,
}

impl DiagnosticOutcome {
    fn render(&self) -> String {
        let mut output = format!(
            "initial_tick={} first_basis_ticks={:?}\n",
            self.initial.running_updates(),
            self.first_bases
                .each_ref()
                .map(|basis| basis.as_ref().map(|basis| basis.parts().tick))
        );
        for job in &self.jobs {
            writeln!(
                output,
                "job run={} basis={:?} policy={:?} seat={:?} actors={:?} memory={} history={} incoming={:?} failed_target=<core-private> stop={:?} rng_key={:?} rng={}->{} output={:?} outgoing={:?} plan={} certificate={:?} contact={} work={}",
                job.run_tick,
                job.basis_tick,
                job.policy,
                job.recipient,
                job.actors,
                job.memory,
                job.history,
                job.incoming,
                job.settled_stop_tick,
                job.rng_key,
                job.rng_before,
                job.rng_after,
                job.output,
                job.outgoing,
                job.plan,
                job.certificate,
                job.contact,
                job.work
            )
            .unwrap();
        }
        for action in &self.actions {
            writeln!(
                output,
                "action run={} seat={:?} {}",
                action.run_tick, action.seat, action.action
            )
            .unwrap();
        }
        for strategic in &self.strategic {
            writeln!(
                output,
                "strategic run={} cell={:?} intent={:?}",
                strategic.run_tick, strategic.cell, strategic.intent
            )
            .unwrap();
        }
        for event in &self.events {
            writeln!(output, "event run={} {}", event.run_tick, event.event).unwrap();
        }
        for (tick, seat, admission) in &self.admissions {
            writeln!(
                output,
                "admission run={tick} seat={seat:?} trace={admission:?}"
            )
            .unwrap();
        }
        output
    }
}

#[test]
#[ignore = "manual ordered Hard67 approach diagnostic; run with --ignored --nocapture"]
fn hard67_seed53_first_common_basis_and_approach_timeline_are_captured() {
    let standard = run(CpuDifficulty::Standard);
    let hard = run(CpuDifficulty::Hard);

    assert_eq!(standard.initial, hard.initial);
    assert_eq!(standard.first_bases, hard.first_bases);
    assert!(standard.first_bases.iter().all(Option::is_some));
    assert!(hard.jobs.iter().any(|job| job.recipient == PRIORITY));
    assert!(standard.jobs.iter().any(|job| job.recipient == PRIORITY));

    eprintln!("STANDARD62 HARD-APPROACH DIAGNOSTIC\n{}", standard.render());
    eprintln!("HARD67 HARD-APPROACH DIAGNOSTIC\n{}", hard.render());
}

fn run(difficulty: CpuDifficulty) -> DiagnosticOutcome {
    let profile = match difficulty {
        CpuDifficulty::Standard => grenadiers_core::room::CpuProfile::RECOVERY_V62_STANDARD,
        CpuDifficulty::Hard => grenadiers_core::room::CpuProfile::HARD_V67,
        CpuDifficulty::Easy => unreachable!(),
    };
    let seeds: [[u8; 32]; 3] =
        std::array::from_fn(|index| [CPU_SEED_SET.wrapping_add(u8::try_from(index).unwrap()); 32]);
    let mut recording = RecordingRosterRuntime::new_for_evaluation(
        configuration(difficulty),
        grenadiers_core::room::RoomSettings::DEFAULT
            .with_enabled_items(grenadiers_core::game::EnabledItems::ALL)
            .with_bomb_capacity_drop_multiplier(
                grenadiers_core::room::BombCapacityDropMultiplier::DEFAULT,
            ),
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(ARENA_SEED);
            Ok(())
        }),
        [SIMULATION_ROOT_SEED; 32],
        CPU_SEATS
            .into_iter()
            .enumerate()
            .map(|(index, seat)| (seat, difficulty, seeds[index]))
            .collect(),
    )
    .unwrap();
    support::start_round(&mut recording).unwrap();
    let mut runtime = recording.runtime().clone();
    runtime.enable_evaluation_trace_for_test();
    let notice = runtime.current_notice().unwrap();
    let initial = runtime.state_for_test().unwrap().clone();
    let room_instance = runtime.identity().registry_serial;
    let planner = CpuPlanner::without_workers();
    let cpus = CPU_SEATS
        .into_iter()
        .enumerate()
        .map(|(index, seat)| {
            CpuController::new(planner.clone(), seat, room_instance, seeds[index], profile)
        })
        .collect::<Vec<_>>();
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(ARENA_SEED);
        Ok(())
    });
    let mut driver = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        cpus,
    );
    driver.publish(notice);

    let world = WorldId {
        room_instance,
        physical_round: 1,
    };
    let mut output = DiagnosticOutcome {
        initial,
        first_bases: std::array::from_fn(|_| None),
        jobs: Vec::new(),
        actions: Vec::new(),
        strategic: Vec::new(),
        events: Vec::new(),
        admissions: Vec::new(),
    };
    let mut previous_summary = None;
    let mut previous_admissions = std::array::from_fn(|_| AdmissionTrace::default());
    for _ in 0..TICKS {
        if driver.runtime.state_for_test().unwrap().result().is_some() {
            break;
        }
        let running = driver.runtime.state_for_test().unwrap().running_updates();
        if running.is_multiple_of(12) {
            driver
                .runtime
                .action(SeatId::One, 1, RosterAction::Movement(None))
                .unwrap();
        }
        drain_actions(&mut driver, &mut output);
        planner.pump_for_test_witnessing(usize::MAX, |witness| {
            capture_job(running, witness, &mut output)
        });
        driver.tick(Instant::now()).unwrap();
        drain_actions(&mut driver, &mut output);
        capture_step(&driver, world, &mut previous_summary, &mut output);
        capture_admissions(&driver, &mut previous_admissions, &mut output);
    }
    output
}

fn capture_job(run_tick: u64, witness: PlannedResultWitness<'_>, output: &mut DiagnosticOutcome) {
    let parts = witness.basis.observation.parts();
    if let Some(index) = CPU_SEATS.iter().position(|seat| *seat == parts.recipient) {
        output.first_bases[index].get_or_insert_with(|| witness.basis.observation.clone());
    }
    if parts.recipient != PRIORITY && output.first_bases.iter().all(Option::is_some) {
        return;
    }
    let plan = witness.plan;
    output.jobs.push(JobTrace {
        run_tick,
        basis_tick: parts.tick,
        policy: *witness.basis.policy,
        recipient: parts.recipient,
        actors: parts
            .actors
            .iter()
            .map(|actor| {
                let (alive, cell) = match actor {
                    grenadiers_core::cpu::ObservedActor::Visible(actor) => {
                        (actor.alive, actor.position.cell())
                    }
                    grenadiers_core::cpu::ObservedActor::Hidden { alive, .. } => (*alive, None),
                };
                (actor.seat(), alive, cell)
            })
            .collect(),
        memory: format!(
            "key={:?} route={:?} planted={:?} refuge={:?} goal={:?} previous={:?} recent={:?} deferred={:?}",
            witness.basis.memory.key(),
            witness.basis.memory.route(),
            witness.basis.memory.planted_origin(),
            witness.basis.memory.refuge(),
            witness.basis.memory.goal(),
            witness.basis.memory.previous_cell(),
            witness.basis.memory.recent_cells(),
            witness.basis.memory.deferred_goal()
        ),
        history: format!("{:?}", witness.basis.hazard_memory),
        incoming: witness.basis.memory.strategic_intent(),
        settled_stop_tick: witness.basis.settled_stop_tick,
        rng_key: witness.rng_before.key(),
        rng_before: witness.rng_before.draws(),
        rng_after: witness.basis.rng.draws(),
        output: plan.map(|plan| plan.control.intent),
        outgoing: plan.and_then(|plan| plan.control.next_memory.strategic_intent()),
        plan: format!("{plan:?}"),
        certificate: plan
            .and_then(|plan| plan.execution.as_ref())
            .map(certificate_kind),
        contact: witness.has_contact,
        work: witness.work_spent,
    });
}

fn certificate_kind(certificate: &ExecutionCertificate) -> &'static str {
    match certificate {
        ExecutionCertificate::Route(_) => "route",
        ExecutionCertificate::Plant(_) => "plant",
        ExecutionCertificate::ContactPlant(_) => "contact-plant",
        ExecutionCertificate::Punch(_) => "punch",
        ExecutionCertificate::ReactivePlant(_) => "reactive-plant",
        ExecutionCertificate::ReactiveRoute(_) => "reactive-route",
    }
}

fn drain_actions(driver: &mut Driver, output: &mut DiagnosticOutcome) {
    let run_tick = driver.runtime.state_for_test().unwrap().running_updates();
    for operation in driver.runtime.drain_evaluation_trace_for_test().unwrap() {
        if let EvaluationOperation::CpuAction { seat, action, .. } = operation {
            output.actions.push(ActionTrace {
                run_tick,
                seat,
                action: format!("{action:?}"),
            });
        }
    }
}

fn capture_step(
    driver: &Driver,
    world: WorldId,
    previous: &mut Option<(
        Option<grenadiers_core::game::CellId>,
        grenadiers_core::cpu::policy::IntentDebugSummary,
    )>,
    output: &mut DiagnosticOutcome,
) {
    let state = driver.runtime.state_for_test().unwrap();
    let run_tick = state.running_updates();
    let (_, report) = driver.runtime.last_step_trace_for_test().unwrap();
    for event in &report.events {
        if matches!(
            &event.kind,
            StepEventKind::BombPlaced(_) | StepEventKind::ActorEliminated { .. }
        ) {
            output.events.push(EventTrace {
                run_tick,
                event: format!("{:?}", event.kind),
            });
        }
    }
    let index = CPU_SEATS.iter().position(|seat| *seat == PRIORITY).unwrap();
    let Some(summary) = driver.cpus[index].intent_debug_summary_for(world) else {
        return;
    };
    let cell = state.actors()[PRIORITY.index()].position().cell();
    if previous.as_ref() != Some(&(cell, summary)) {
        output.strategic.push(StrategicTrace {
            run_tick,
            cell,
            intent: summary,
        });
        *previous = Some((cell, summary));
    }
}

fn capture_admissions(
    driver: &Driver,
    previous: &mut [AdmissionTrace; 3],
    output: &mut DiagnosticOutcome,
) {
    let run_tick = driver.runtime.state_for_test().unwrap().running_updates();
    for (index, cpu) in driver.cpus.iter().enumerate() {
        let trace = cpu.admission_trace_for_test();
        if trace != previous[index] {
            output
                .admissions
                .push((run_tick, CPU_SEATS[index], trace.clone()));
            previous[index] = trace;
        }
    }
}
