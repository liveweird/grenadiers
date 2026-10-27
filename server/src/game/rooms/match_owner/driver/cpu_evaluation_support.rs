use std::collections::BTreeSet;
use std::time::Duration;

use grenadiers_core::cpu::{Intent, ObservedOil, ObservedPickup};
use grenadiers_core::game::{CellId, EntityId, Position, RoundResult, SeatId, Terrain, Tick};
use grenadiers_core::room::{CpuProfile, RoomSettings};
use tokio::time::Instant;

use super::super::super::super::cpu::PumpReport;
use super::super::super::super::match_runtime::MatchRuntimeError;
use super::super::super::super::replay::ReplayAction;

#[path = "cpu_evaluation_events.rs"]
mod events;
#[path = "cpu_evaluation_fixture.rs"]
mod fixture;
#[path = "cpu_evaluation_script.rs"]
mod script;
#[path = "cpu_evaluation_support_tests.rs"]
mod tests;

const SOURCE: &str = "generated-v2/test-recording-enabled";
const SCHEDULE: &str = "pump-max32,poll,script,driver-tick";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HumanScript {
    Idle,
    BombRetreat,
}

impl HumanScript {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::BombRetreat => "bomb-retreat",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct EvaluationCase {
    pub(super) id: &'static str,
    pub(super) arena_seed_id: &'static str,
    pub(super) cpu_seed_id: &'static str,
    pub(super) script: HumanScript,
    pub(super) tick_cap: u64,
    arena_seed: [u8; 32],
    cpu_seed: [u8; 32],
}

impl EvaluationCase {
    pub(super) const fn new(
        id: &'static str,
        arena_seed_id: &'static str,
        arena_seed: [u8; 32],
        cpu_seed_id: &'static str,
        cpu_seed: [u8; 32],
        script: HumanScript,
        tick_cap: u64,
    ) -> Self {
        Self {
            id,
            arena_seed_id,
            cpu_seed_id,
            script,
            tick_cap,
            arena_seed,
            cpu_seed,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvaluationFrame {
    pub(super) running_tick: u64,
    pub(super) physical_tick: Tick,
    pub(super) cpu_position: Position,
    pub(super) human_position: Position,
    pub(super) cpu_alive: bool,
    pub(super) human_alive: bool,
    pub(super) exposed_pickups: usize,
    pub(super) blocks: usize,
    pub(super) rng_draws: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ObservedBombPlacement {
    pub(super) tick: Tick,
    pub(super) cell: CellId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct InitialPublicWorld {
    pub(super) terrain: Vec<Terrain>,
    pub(super) blocks: Vec<CellId>,
    pub(super) pickups: Vec<ObservedPickup>,
    pub(super) oil: Vec<ObservedOil>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct PlanningMetrics {
    pub(super) attempts: usize,
    pub(super) delivered: usize,
    pub(super) cancelled: usize,
    pub(super) failed: usize,
    pub(super) dropped: usize,
    pub(super) work: Vec<u32>,
    pub(super) planned_intents: Vec<Intent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvaluationTrace {
    pub(super) case_id: &'static str,
    pub(super) arena_seed_id: &'static str,
    pub(super) cpu_seed_id: &'static str,
    pub(super) script: &'static str,
    pub(super) tick_cap: u64,
    pub(super) profile: CpuProfile,
    pub(super) settings: RoomSettings,
    pub(super) source: &'static str,
    pub(super) schedule: &'static str,
    pub(super) initial_public_world: InitialPublicWorld,
    pub(super) terminal: Option<RoundResult>,
    pub(super) censored: bool,
    pub(super) running_ticks: u64,
    pub(super) cpu_alive: bool,
    pub(super) accepted_movement_actions: usize,
    pub(super) accepted_manual_actions: usize,
    pub(super) accepted_actions: Vec<(Tick, SeatId, ReplayAction)>,
    pub(super) first_observed_cpu_bomb_placed_tick: Option<Tick>,
    pub(super) first_observed_cpu_pickup_tick: Option<Tick>,
    pub(super) first_observed_human_bomb_placement: Option<ObservedBombPlacement>,
    pub(super) visited_cells: Vec<CellId>,
    pub(super) stationary_ticks: u64,
    pub(super) planning: PlanningMetrics,
    pub(super) work_sum: u64,
    pub(super) work_max: u32,
    pub(super) rng_draws: u64,
    pub(super) frames: Vec<EvaluationFrame>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EvaluationRun {
    pub(super) trace: EvaluationTrace,
    /// Test-only wall-time samples. Never included in deterministic trace equality.
    pub(super) planner_times: Vec<Duration>,
}

pub(super) fn default_cases() -> Vec<EvaluationCase> {
    fixture::default_cases()
}

pub(super) fn run(case: EvaluationCase) -> Result<EvaluationRun, MatchRuntimeError> {
    let (mut driver, planner, settings) =
        fixture::running_recording_driver(case.arena_seed, case.cpu_seed);
    let initial = driver.runtime.current_notice()?;
    let initial_world = initial
        .cpu_observation
        .as_ref()
        .map(|observation| InitialPublicWorld {
            terrain: observation.parts().arena.terrain.clone(),
            blocks: observation.parts().arena.blocks.clone(),
            pickups: observation.parts().pickups.clone(),
            oil: observation.parts().oil.clone(),
        })
        .ok_or(MatchRuntimeError::Publication)?;
    let initial_position = driver
        .runtime
        .sim()
        .ok_or(MatchRuntimeError::Publication)?
        .state
        .actors()[SeatId::Two.index()]
    .position();
    driver.publish(initial);
    let mut accumulator = Accumulator::new(case, settings, initial_world, initial_position);
    let mut manual_id = 1_u32;

    while accumulator.running_ticks < case.tick_cap && accumulator.terminal.is_none() {
        accumulator.add_pump(planner.pump_for_test(usize::MAX));
        driver.poll_cpu_result()?;
        script::apply(
            &mut driver,
            case.script,
            accumulator.running_ticks,
            &mut manual_id,
        )?;
        driver.tick(Instant::now())?;
        accumulator.observe(&driver);
    }
    Ok(accumulator.finish(&driver))
}

struct Accumulator {
    case: EvaluationCase,
    settings: RoomSettings,
    initial_world: InitialPublicWorld,
    running_ticks: u64,
    terminal: Option<RoundResult>,
    visited: BTreeSet<CellId>,
    stationary_ticks: u64,
    previous_position: Option<Position>,
    first_bomb: Option<Tick>,
    first_pickup: Option<Tick>,
    first_human_bomb: Option<ObservedBombPlacement>,
    planning: PlanningMetrics,
    planner_times: Vec<Duration>,
    frames: Vec<EvaluationFrame>,
}

impl Accumulator {
    fn new(
        case: EvaluationCase,
        settings: RoomSettings,
        initial_world: InitialPublicWorld,
        initial_position: Position,
    ) -> Self {
        let mut visited = BTreeSet::new();
        visited.extend(initial_position.cell());
        Self {
            case,
            settings,
            initial_world,
            running_ticks: 0,
            terminal: None,
            visited,
            stationary_ticks: 0,
            previous_position: Some(initial_position),
            first_bomb: None,
            first_pickup: None,
            first_human_bomb: None,
            planning: PlanningMetrics::default(),
            planner_times: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn add_pump(&mut self, report: PumpReport) {
        self.planning.attempts += report.dequeued;
        self.planning.delivered += report.delivered;
        self.planning.cancelled += report.cancelled;
        self.planning.failed += report.failed;
        self.planning.dropped += report.dropped;
        self.planning.work.extend(report.work);
        self.planning.planned_intents.extend(report.planned_intents);
        self.planner_times.extend(report.planner_times);
    }

    fn observe(&mut self, driver: &super::super::Driver) {
        self.running_ticks += 1;
        let state = &driver
            .runtime
            .sim()
            .expect("running evaluation has state")
            .state;
        let cpu = &state.actors()[SeatId::Two.index()];
        let human = &state.actors()[SeatId::One.index()];
        let position = cpu.position();
        self.visited.extend(position.cell());
        self.stationary_ticks += u64::from(self.previous_position == Some(position));
        self.previous_position = Some(position);
        self.observe_public_events(driver, cpu.id(), human.id());
        self.terminal = state.result();
        self.frames.push(EvaluationFrame {
            running_tick: self.running_ticks,
            physical_tick: state.tick(),
            cpu_position: position,
            human_position: human.position(),
            cpu_alive: cpu.alive(),
            human_alive: human.alive(),
            exposed_pickups: state.world().exposed_pickups().len(),
            blocks: state.world().arena().blocks().len(),
            rng_draws: driver.cpu.as_ref().map_or(0, |cpu| cpu.rng_draws()),
        });
    }

    fn finish(self, driver: &super::super::Driver) -> EvaluationRun {
        let mut movement = 0;
        let mut manual = 0;
        let accepted_actions = driver
            .runtime
            .replay_journal()
            .accepted_actions()
            .collect::<Vec<_>>();
        for (_, seat, action) in accepted_actions.iter().copied() {
            if seat != SeatId::Two {
                continue;
            }
            match action {
                ReplayAction::Movement(_) => movement += 1,
                ReplayAction::Bomb(_) => {
                    manual += 1;
                }
                ReplayAction::Equipment(_) => manual += 1,
                _ => {}
            }
        }
        let cpu_alive = driver
            .runtime
            .sim()
            .is_some_and(|sim| sim.state.actors()[SeatId::Two.index()].alive());
        let work_sum = self
            .planning
            .work
            .iter()
            .map(|value| u64::from(*value))
            .sum();
        let work_max = self.planning.work.iter().copied().max().unwrap_or(0);
        let rng_draws = driver.cpu.as_ref().map_or(0, |cpu| cpu.rng_draws());
        EvaluationRun {
            trace: EvaluationTrace {
                case_id: self.case.id,
                arena_seed_id: self.case.arena_seed_id,
                cpu_seed_id: self.case.cpu_seed_id,
                script: self.case.script.name(),
                tick_cap: self.case.tick_cap,
                profile: CpuProfile::CURRENT,
                settings: self.settings,
                source: SOURCE,
                schedule: SCHEDULE,
                initial_public_world: self.initial_world,
                terminal: self.terminal,
                censored: self.terminal.is_none(),
                running_ticks: self.running_ticks,
                cpu_alive,
                accepted_movement_actions: movement,
                accepted_manual_actions: manual,
                accepted_actions,
                first_observed_cpu_bomb_placed_tick: self.first_bomb,
                first_observed_cpu_pickup_tick: self.first_pickup,
                first_observed_human_bomb_placement: self.first_human_bomb,
                visited_cells: self.visited.into_iter().collect(),
                stationary_ticks: self.stationary_ticks,
                planning: self.planning,
                work_sum,
                work_max,
                rng_draws,
                frames: self.frames,
            },
            planner_times: self.planner_times,
        }
    }

    fn observe_public_events(
        &mut self,
        driver: &super::super::Driver,
        cpu_id: EntityId,
        human_id: EntityId,
    ) {
        if let Some(frame) = driver.last.frame(SeatId::Two) {
            let (bomb, pickup) = events::cpu_event_ticks(&frame, cpu_id);
            if self.first_bomb.is_none() {
                self.first_bomb = bomb;
            }
            if self.first_pickup.is_none() {
                self.first_pickup = pickup;
            }
        }
        if self.first_human_bomb.is_none() {
            self.first_human_bomb = driver
                .last
                .frame(SeatId::One)
                .and_then(|frame| events::bomb_placement(&frame, human_id));
        }
    }
}
