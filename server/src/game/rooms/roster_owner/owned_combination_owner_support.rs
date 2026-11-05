//! Curated public setup; no generated-world or restored-match provenance is implied.
use std::{collections::BTreeSet, sync::Arc};

#[path = "owned_combination_runtime_support.rs"]
mod runtime_support;

use grenadiers_core::{
    cpu::{ManualIntent, WorldId, policy::StrategicTarget},
    game::{
        BlastKind, BombTrigger, CellId, Direction, EntityId, InputAction, PickupKind, SeatId,
        SeatInput, StepEventKind, Tick,
    },
    room::CpuProfile,
};
use tokio::{sync::watch, time::Instant};

use super::{RosterOwnerNotice, driver::Driver};
use crate::game::rooms::{
    RandomFill,
    cpu::{AdmissionTrace, CpuController, CpuPlanner},
    roster_runtime::{RosterAction, RosterRuntime},
};

pub(super) const CPU: SeatId = SeatId::Two;
const RIVAL: SeatId = SeatId::Three;
const INSTANCE: u64 = 4_067;
pub(super) const ADDITIONAL_BLOCK: CellId = CellId::new(7, 3);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PlantEvent {
    pub id: EntityId,
    pub tick: Tick,
    pub cell: CellId,
    pub detonates_at: Tick,
    pub clears_at: Tick,
    pub range: u16,
    pub blast: BlastKind,
}

#[derive(Debug)]
pub(super) struct Outcome {
    pub profile: CpuProfile,
    pub pickups: Vec<(Tick, PickupKind)>,
    pub plants: Vec<PlantEvent>,
    pub simultaneous_owned: usize,
    pub additional_intact_at_second: bool,
    pub additional_destroyed_at: Option<Tick>,
    pub additional_destroyed_by: Option<EntityId>,
    pub detonations: Vec<(Tick, EntityId)>,
    pub selected_rival_at_second: bool,
    pub route_profile_support_at_second: Option<[bool; 5]>,
    pub survived_both_windows: bool,
    pub settled_progress_after_windows: bool,
    pub manual_attempts_after_first: usize,
    pub delivered: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub dropped: usize,
    pub max_work: u32,
    pub admission: AdmissionTrace,
    pub dequeued: usize,
    pub queue_drained: bool,
}

pub(super) fn run(profile: CpuProfile) -> Outcome {
    let (mut owner, planner, pickups) = driver(profile);
    let epoch = owner.runtime.lifecycle().round_attempt();
    let world = WorldId {
        room_instance: INSTANCE,
        physical_round: epoch,
    };
    let mut plants: Vec<PlantEvent> = Vec::new();
    let mut simultaneous_owned = 0;
    let mut additional_intact_at_second = false;
    let mut additional_destroyed_at = None;
    let mut additional_destroyed_by = None;
    let mut detonations = Vec::new();
    let mut selected_rival_at_second = false;
    let mut route_profile_support_at_second = None;
    let mut manual_attempts_after_first = 0;
    let mut delivered = 0;
    let mut failed = 0;
    let mut cancelled = 0;
    let mut dropped = 0;
    let mut max_work = 0;
    let mut dequeued = 0;
    let mut windows_complete_at = None;
    let mut position_after_windows = None;
    let mut settled_progress_after_windows = false;

    for _ in 0..900 {
        if let Some(first) = plants.first().copied() {
            let offset = owner.runtime.lifecycle().tick().0 - first.tick.0;
            let movement = match offset {
                0 => Some(Some(Direction::East)),
                12 => Some(Some(Direction::North)),
                27 => Some(Some(Direction::North)),
                36 => Some(None),
                _ => None,
            };
            if let Some(direction) = movement {
                owner
                    .runtime
                    .action(RIVAL, epoch, RosterAction::Movement(direction))
                    .unwrap();
            }
        }
        let report = planner.pump_for_test(usize::MAX);
        if !plants.is_empty() {
            manual_attempts_after_first += report
                .planned_intents
                .iter()
                .filter(|intent| intent.manual == Some(ManualIntent::Bomb))
                .count();
        }
        delivered += report.delivered;
        dequeued += report.dequeued;
        failed += report.failed;
        cancelled += report.cancelled;
        dropped += report.dropped;
        max_work = max_work.max(report.work.into_iter().max().unwrap_or(0));
        let block_was_intact = owner
            .runtime
            .state_for_test()
            .unwrap()
            .world()
            .arena()
            .blocks()
            .contains(&ADDITIONAL_BLOCK);
        owner.tick(Instant::now()).unwrap();
        let state = owner.runtime.state_for_test().unwrap();
        let owned = state
            .world()
            .bombs()
            .values()
            .filter(|bomb| bomb.owner() == EntityId(2))
            .count();
        simultaneous_owned = simultaneous_owned.max(owned);
        let (_, physics) = owner.runtime.last_step_trace_for_test().unwrap();
        for event in &physics.events {
            if let StepEventKind::BombPlaced(bomb) = &event.kind
                && bomb.owner == EntityId(2)
                && plants.iter().all(|plant: &PlantEvent| plant.id != bomb.id)
            {
                let BombTrigger::Timed { detonates_at } = bomb.trigger else {
                    panic!("controlled combination witness uses timed bombs")
                };
                plants.push(PlantEvent {
                    id: bomb.id,
                    tick: event.tick,
                    cell: bomb.cell,
                    detonates_at,
                    clears_at: Tick(detonates_at.0 + state.rules().flame_ticks()),
                    range: bomb.range,
                    blast: bomb.blast,
                });
                if plants.len() == 2 {
                    additional_intact_at_second =
                        state.world().arena().blocks().contains(&ADDITIONAL_BLOCK);
                    selected_rival_at_second = owner.cpus[0]
                        .intent_debug_summary_for(world)
                        .is_some_and(|summary| {
                            summary.target == StrategicTarget::Opponent { seat: RIVAL }
                        });
                    route_profile_support_at_second = Some([
                        owner.cpus[0]
                            .committed_route_support_for_profile_test(CpuProfile::HARD_V65)
                            .unwrap_or(false),
                        owner.cpus[0]
                            .committed_route_support_for_profile_test(CpuProfile::HARD_V66)
                            .unwrap_or(false),
                        owner.cpus[0]
                            .committed_route_support_for_profile_test(CpuProfile::HARD_V67)
                            .unwrap_or(false),
                        owner.cpus[0]
                            .committed_route_support_for_profile_test(CpuProfile::HARD_V69)
                            .unwrap_or(false),
                        owner.cpus[0]
                            .committed_route_support_for_profile_test(CpuProfile::HARD_V70)
                            .unwrap_or(false),
                    ]);
                    windows_complete_at = plants.iter().map(|plant| plant.clears_at).max();
                }
            }
            if let StepEventKind::BombDetonated(bomb) = &event.kind {
                detonations.push((event.tick, bomb.id));
            }
        }
        if additional_destroyed_at.is_none()
            && block_was_intact
            && !state.world().arena().blocks().contains(&ADDITIONAL_BLOCK)
        {
            additional_destroyed_at = Some(state.tick());
            let detonated: Vec<_> = physics
                .events
                .iter()
                .filter_map(|event| match &event.kind {
                    StepEventKind::BombDetonated(bomb) => Some(bomb.id),
                    _ => None,
                })
                .collect();
            additional_destroyed_by = (detonated.len() == 1).then(|| detonated[0]);
        }
        if windows_complete_at.is_some_and(|tick| state.tick() > tick) {
            let cell = state.actor(CPU).unwrap().position().cell();
            let after = *position_after_windows.get_or_insert(cell);
            settled_progress_after_windows |=
                owner.cpu_stationary[CPU.index()] && cell.is_some() && cell != after;
            if settled_progress_after_windows {
                break;
            }
        }
    }
    for controller in &mut owner.cpus {
        controller.cancel();
    }
    let final_report = planner.pump_for_test(usize::MAX);
    dequeued += final_report.dequeued;
    delivered += final_report.delivered;
    failed += final_report.failed;
    cancelled += final_report.cancelled;
    dropped += final_report.dropped;
    max_work = max_work.max(final_report.work.into_iter().max().unwrap_or(0));
    let queue_drained = planner.pump_for_test(usize::MAX).dequeued == 0
        && owner
            .cpus
            .iter()
            .all(|controller| !controller.has_pending());
    let state = owner.runtime.state_for_test().unwrap();
    let survived_both_windows = windows_complete_at.is_some_and(|tick| state.tick() > tick)
        && state.actor(CPU).unwrap().alive();
    Outcome {
        profile,
        pickups,
        plants,
        simultaneous_owned,
        additional_intact_at_second,
        additional_destroyed_at,
        additional_destroyed_by,
        detonations,
        selected_rival_at_second,
        route_profile_support_at_second,
        survived_both_windows,
        settled_progress_after_windows,
        manual_attempts_after_first,
        delivered,
        failed,
        cancelled,
        dropped,
        max_work,
        admission: owner.cpus[0].admission_trace_for_test(),
        dequeued,
        queue_drained,
    }
}

fn driver(profile: CpuProfile) -> (Driver, CpuPlanner, Vec<(Tick, PickupKind)>) {
    let (mut runtime, controlled, pickups) = running_scene(profile);
    let notice = runtime.install_running_scene_for_test(controlled).unwrap();
    let planner = CpuPlanner::without_workers();
    let controller = CpuController::new(planner.clone(), CPU, INSTANCE, [67; 32], profile);
    let (notices, receiver) = watch::channel(RosterOwnerNotice::Publication(notice.clone()));
    drop(receiver);
    let random_fill: Arc<RandomFill> = Arc::new(|bytes| {
        bytes.fill(67);
        Ok(())
    });
    let mut owner = Driver::new(
        runtime,
        notice.clone(),
        notices,
        Instant::now() + std::time::Duration::from_secs(300),
        random_fill,
        vec![controller],
    );
    owner.publish(notice);
    (owner, planner, pickups)
}

fn running_scene(
    profile: CpuProfile,
) -> (
    RosterRuntime,
    grenadiers_core::game::State,
    Vec<(Tick, PickupKind)>,
) {
    let (mut runtime, settings) = runtime_support::runtime(profile, INSTANCE);
    for seat in [SeatId::One, RIVAL] {
        let admission = runtime.prepare_join(seat).unwrap();
        runtime.commit_admission(admission);
        runtime
            .action(
                seat,
                1,
                RosterAction::Ready {
                    ready: true,
                    settings_revision: 1,
                },
            )
            .unwrap();
    }
    for _ in 0..grenadiers_core::room::COUNTDOWN_TICKS {
        runtime.settle_boundary([None; 4]).unwrap();
    }
    let blocks = BTreeSet::from([ADDITIONAL_BLOCK]);
    let mut controlled = super::refuge_handoff_physical_support::controlled_state_with_blocks(
        runtime.state_for_test().unwrap(),
        settings,
        blocks,
    );
    let mut pickups = Vec::new();
    for update in 0..49 {
        let action = if update < 48 {
            InputAction::Press(Direction::North)
        } else {
            InputAction::Clear
        };
        let report = controlled.step(&[SeatInput { seat: CPU, action }]).unwrap();
        pickups.extend(
            report
                .events
                .into_iter()
                .filter_map(|event| match event.kind {
                    StepEventKind::PickupCollected {
                        seat: CPU, kind, ..
                    } => Some((event.tick, kind)),
                    _ => None,
                }),
        );
        runtime.settle_boundary([None; 4]).unwrap();
    }
    assert!(
        pickups
            .iter()
            .any(|(_, kind)| *kind == PickupKind::BombCapacity)
    );
    assert_eq!(controlled.actor(CPU).unwrap().bomb_capacity(), 2);
    (runtime, controlled, pickups)
}
