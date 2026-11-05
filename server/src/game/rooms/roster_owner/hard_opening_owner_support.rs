use std::{collections::BTreeSet, sync::Arc};

use grenadiers_core::{
    cpu::Observation,
    game::{BombView, CellId, EntityId, SeatId, State, StepEventKind, Tick},
    room::{CpuDifficulty, CpuProfile, RoomSettings},
};
use tokio::{sync::watch, time::Instant};

use super::{
    RosterOwnerNotice,
    driver::Driver,
    live_cpu_evaluation::{configuration, start_round},
    live_cpu_evaluation_types::CPU_SEATS,
};
use crate::{
    game::rooms::{
        RandomFill,
        cpu::{AdmissionTrace, CpuController, CpuPlanner, PumpReport},
        replay_v38, replay_v39,
        roster_runtime::{EvaluationOperation, RosterAction, RosterRuntime},
    },
    round_seed::RoundSeedSource,
};

const ARENA_SEED: u8 = 53;
const CPU_SEED_SET: u8 = 71;
const SIMULATION_ROOT_SEED: u8 = 113;
const TICKS: usize = 720;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ActualBomb {
    pub seat: SeatId,
    pub id: EntityId,
    pub accepted_running_tick: u64,
    pub placed_at: Tick,
    pub cell: CellId,
    pub detonates_at: Tick,
    pub clears_at: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ProductiveBomb {
    pub bomb: ActualBomb,
    pub destroyed_block: CellId,
    pub destroyed_at: Tick,
    pub blocks_destroyed: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct QueueLedger {
    pub dequeued: usize,
    pub delivered: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub dropped: usize,
    pub total_work: u64,
    pub max_work: u32,
}

impl QueueLedger {
    fn merge(&mut self, report: PumpReport) {
        self.dequeued += report.dequeued;
        self.delivered += report.delivered;
        self.failed += report.failed;
        self.cancelled += report.cancelled;
        self.dropped += report.dropped;
        self.total_work += report
            .work
            .iter()
            .map(|value| u64::from(*value))
            .sum::<u64>();
        self.max_work = self
            .max_work
            .max(report.work.into_iter().max().unwrap_or(0));
    }
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub initial: State,
    pub first_bases: [Option<Observation>; 3],
    pub accepted_bombs: Vec<(u64, SeatId)>,
    pub placed_bombs: usize,
    pub first_priority_bomb: Option<ActualBomb>,
    pub productive: Option<ProductiveBomb>,
    pub survived_window: bool,
    pub completed_escape: bool,
    pub movement_after_window: bool,
    pub ledger: QueueLedger,
    pub admissions: [AdmissionTrace; 3],
    pub rng_draws: [u64; 3],
    pub queue_drained: bool,
}

pub(super) fn run(profile: CpuProfile) -> Outcome {
    let mut runtime = runtime(profile);
    runtime.enable_evaluation_trace_for_test();
    let notice = runtime.current_notice().unwrap();
    let initial = runtime.state_for_test().unwrap().clone();
    let room_instance = runtime.identity().registry_serial;
    let seeds = cpu_seeds();
    let planner = CpuPlanner::without_workers();
    let cpus = CPU_SEATS
        .into_iter()
        .enumerate()
        .map(|(index, seat)| {
            CpuController::new(planner.clone(), seat, room_instance, seeds[index], profile)
        })
        .collect();
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

    let mut first_bases = std::array::from_fn(|_| None);
    let mut accepted = Vec::new();
    let mut accepted_history = Vec::new();
    let mut placed = Vec::new();
    let mut first_priority_bomb = None;
    let mut productive = None;
    let mut post_window_cell = None;
    let mut survived_window = false;
    let mut completed_escape = false;
    let mut movement_after_window = false;
    let mut ledger = QueueLedger::default();

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
        drain_bomb_actions(&mut driver, &mut accepted, &mut accepted_history);
        ledger.merge(planner.pump_for_test_witnessing(usize::MAX, |witness| {
            let recipient = witness.basis.observation.parts().recipient;
            let index = CPU_SEATS
                .iter()
                .position(|seat| *seat == recipient)
                .unwrap();
            first_bases[index].get_or_insert_with(|| witness.basis.observation.clone());
        }));
        let blocks_before = driver
            .runtime
            .state_for_test()
            .unwrap()
            .world()
            .arena()
            .blocks()
            .clone();
        driver.tick(Instant::now()).unwrap();
        drain_bomb_actions(&mut driver, &mut accepted, &mut accepted_history);
        capture_step(
            &driver,
            &mut accepted,
            &mut placed,
            &mut first_priority_bomb,
            &mut productive,
            blocks_before,
        );
        if let Some(bomb) = productive {
            let state = driver.runtime.state_for_test().unwrap();
            if state.tick() > bomb.bomb.clears_at {
                let actor = state.actor(bomb.bomb.seat).unwrap();
                survived_window |= actor.alive();
                completed_escape |=
                    actor.alive() && actor.position().cell() != Some(bomb.bomb.cell);
                let cell = actor.position().cell();
                let first = *post_window_cell.get_or_insert(cell);
                movement_after_window |= cell.is_some() && cell != first;
                if movement_after_window && first_bases.iter().all(Option::is_some) {
                    break;
                }
            }
        }
    }
    for cpu in &mut driver.cpus {
        cpu.cancel();
    }
    ledger.merge(planner.pump_for_test(usize::MAX));
    let queue_drained = planner.pump_for_test(usize::MAX).dequeued == 0
        && driver.cpus.iter().all(|cpu| !cpu.has_pending());
    Outcome {
        initial,
        first_bases,
        accepted_bombs: accepted_history,
        placed_bombs: placed.len(),
        first_priority_bomb,
        productive,
        survived_window,
        completed_escape,
        movement_after_window,
        ledger,
        admissions: std::array::from_fn(|index| driver.cpus[index].admission_trace_for_test()),
        rng_draws: std::array::from_fn(|index| driver.cpus[index].rng_draws()),
        queue_drained,
    }
}

fn drain_bomb_actions(
    driver: &mut Driver,
    accepted: &mut Vec<(u64, SeatId)>,
    history: &mut Vec<(u64, SeatId)>,
) {
    let tick = driver.runtime.state_for_test().unwrap().running_updates();
    for operation in driver.runtime.drain_evaluation_trace_for_test().unwrap() {
        if let EvaluationOperation::CpuAction {
            seat,
            action: RosterAction::Bomb(_),
            ..
        } = operation
        {
            accepted.push((tick, seat));
            history.push((tick, seat));
        }
    }
}

fn capture_step(
    driver: &Driver,
    accepted: &mut Vec<(u64, SeatId)>,
    placed: &mut Vec<ActualBomb>,
    first_priority_bomb: &mut Option<ActualBomb>,
    productive: &mut Option<ProductiveBomb>,
    blocks_before: BTreeSet<CellId>,
) {
    let state = driver.runtime.state_for_test().unwrap();
    let (_, report) = driver.runtime.last_step_trace_for_test().unwrap();
    for event in &report.events {
        if let StepEventKind::BombPlaced(bomb) = &event.kind {
            let seat = state
                .actors()
                .iter()
                .find(|actor| actor.id() == bomb.owner)
                .unwrap()
                .seat();
            let index = accepted
                .iter()
                .rposition(|(_, candidate)| *candidate == seat)
                .unwrap();
            let (accepted_running_tick, _) = accepted.remove(index);
            let grenadiers_core::game::BombTrigger::Timed { detonates_at } = bomb.trigger else {
                continue;
            };
            let placed_bomb = ActualBomb {
                seat,
                id: bomb.id,
                accepted_running_tick,
                placed_at: bomb.placed_at,
                cell: bomb.cell,
                detonates_at,
                clears_at: Tick(detonates_at.0 + state.rules().flame_ticks()),
            };
            if seat == SeatId::Four {
                first_priority_bomb.get_or_insert(placed_bomb);
            }
            placed.push(placed_bomb);
        }
    }
    let detonated: Vec<BombView> = report
        .events
        .iter()
        .filter_map(|event| match &event.kind {
            StepEventKind::BombDetonated(bomb) => Some(*bomb),
            _ => None,
        })
        .collect();
    let blocks_after = state.world().arena().blocks();
    let removed: Vec<_> = blocks_before.difference(blocks_after).copied().collect();
    if productive.is_none() {
        for destroyed_block in &removed {
            let candidates: Vec<_> = detonated
                .iter()
                .filter(|view| possibly_reaches(*destroyed_block, view))
                .copied()
                .collect();
            if let [view] = candidates.as_slice()
                && let Some(bomb) = placed.iter().find(|bomb| bomb.id == view.id)
                && bomb.seat == SeatId::Four
            {
                *productive = Some(ProductiveBomb {
                    bomb: *bomb,
                    destroyed_block: *destroyed_block,
                    destroyed_at: state.tick(),
                    blocks_destroyed: removed.len(),
                });
                break;
            }
        }
    }
}

fn possibly_reaches(block: CellId, view: &BombView) -> bool {
    u32::from(block.column.abs_diff(view.cell.column))
        + u32::from(block.row.abs_diff(view.cell.row))
        <= u32::from(view.range)
}

fn cpu_seeds() -> [[u8; 32]; 3] {
    std::array::from_fn(|index| [CPU_SEED_SET.wrapping_add(index as u8); 32])
}

fn runtime(profile: CpuProfile) -> RosterRuntime {
    let settings = RoomSettings::DEFAULT
        .with_enabled_items(grenadiers_core::game::EnabledItems::ALL)
        .with_bomb_capacity_drop_multiplier(
            grenadiers_core::room::BombCapacityDropMultiplier::DEFAULT,
        );
    let seeds = cpu_seeds();
    let generation = || {
        RoundSeedSource::with_entropy(|bytes| {
            bytes.fill(ARENA_SEED);
            Ok(())
        })
    };
    let cpus = || {
        CPU_SEATS
            .into_iter()
            .enumerate()
            .map(|(index, seat)| (seat, CpuDifficulty::Hard, seeds[index]))
            .collect()
    };
    match profile {
        CpuProfile::HARD_V69 => {
            let mut recording = replay_v38::RecordingRosterRuntime::new_for_evaluation(
                configuration(CpuDifficulty::Hard),
                settings,
                generation(),
                [SIMULATION_ROOT_SEED; 32],
                cpus(),
            )
            .unwrap();
            start_round_v38(&mut recording);
            recording.runtime().clone()
        }
        CpuProfile::HARD_V70 => {
            let mut recording = replay_v39::RecordingRosterRuntime::new_for_evaluation(
                configuration(CpuDifficulty::Hard),
                settings,
                generation(),
                [SIMULATION_ROOT_SEED; 32],
                cpus(),
            )
            .unwrap();
            start_round(&mut recording).unwrap();
            recording.runtime().clone()
        }
        _ => panic!("opening witness is closed to Hard69 and Hard70"),
    }
}

fn start_round_v38(recording: &mut replay_v38::RecordingRosterRuntime) {
    recording.admit(SeatId::One, false).unwrap();
    recording
        .human_action(
            SeatId::One,
            1,
            replay_v38::ReplayAction::Ready {
                ready: true,
                settings_revision: 1,
            },
        )
        .unwrap();
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        recording.settle([None; 4]).unwrap();
    }
    recording.publish_flush().unwrap();
}
