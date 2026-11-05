use std::collections::BTreeSet;

use grenadiers_core::{
    cpu::WorldId,
    game::{CellId, State, StepEventKind, StepReport},
};

use super::super::cpu::{AdmissionTrace, CpuController};
use super::live_cpu_evaluation_types::{
    AdmissionMetrics, EventMetrics, IntentMetrics, MobilityMetrics,
};

#[derive(Clone, Debug, Default)]
struct MobilityTracker {
    visited: BTreeSet<CellId>,
    previous: Option<CellId>,
    stationary_span: u64,
    output: MobilityMetrics,
}

#[derive(Clone, Debug)]
pub(super) struct MetricsCollector {
    initial_blocks: usize,
    minimum_blocks: usize,
    mobility: [MobilityTracker; 4],
    events: EventMetrics,
    intents: [IntentMetrics; 3],
}

impl MetricsCollector {
    pub(super) fn new(state: &State) -> Self {
        let blocks = state.world().arena().blocks().len();
        let mut collector = Self {
            initial_blocks: blocks,
            minimum_blocks: blocks,
            mobility: std::array::from_fn(|_| MobilityTracker::default()),
            events: EventMetrics::default(),
            intents: std::array::from_fn(|_| IntentMetrics::default()),
        };
        collector.observe_positions(state);
        collector
    }

    pub(super) fn observe(
        &mut self,
        state: &State,
        report: &StepReport,
        cpus: &[CpuController],
        world: WorldId,
    ) {
        self.minimum_blocks = self
            .minimum_blocks
            .min(state.world().arena().blocks().len());
        self.observe_positions(state);
        for event in &report.events {
            match event.kind {
                StepEventKind::BombPlaced(bomb) => {
                    if let Some(index) = state
                        .actors()
                        .iter()
                        .position(|actor| actor.id() == bomb.owner)
                    {
                        self.events.bomb_placed[index] += 1;
                    }
                }
                StepEventKind::BombDetonated(bomb) => {
                    if let Some(index) = state
                        .actors()
                        .iter()
                        .position(|actor| actor.id() == bomb.owner)
                    {
                        self.events.bomb_detonated[index] += 1;
                    }
                }
                StepEventKind::PickupCollected { seat, .. } => {
                    self.events.pickup_collected[seat.index()] += 1;
                }
                StepEventKind::ActorEliminated { seat, .. } => {
                    self.events.actor_eliminated[seat.index()] += 1;
                }
                _ => {}
            }
        }
        for (index, cpu) in cpus.iter().enumerate() {
            let Some(summary) = cpu.intent_debug_summary_for(world) else {
                continue;
            };
            *self.intents[index]
                .phases
                .entry(format!("{:?}", summary.phase))
                .or_default() += 1;
            *self.intents[index]
                .targets
                .entry(format!("{:?}", summary.target))
                .or_default() += 1;
        }
    }

    fn observe_positions(&mut self, state: &State) {
        for (index, actor) in state.actors().iter().enumerate() {
            let cell = actor.position().cell();
            if let Some(cell) = cell {
                self.mobility[index].visited.insert(cell);
            }
            let tracker = &mut self.mobility[index];
            match tracker.previous {
                Some(previous) if Some(previous) == cell => {
                    tracker.output.stationary_updates += 1;
                    tracker.stationary_span += 1;
                    tracker.output.longest_stationary_span = tracker
                        .output
                        .longest_stationary_span
                        .max(tracker.stationary_span);
                }
                Some(_) => {
                    tracker.output.cell_transitions += 1;
                    tracker.stationary_span = 0;
                }
                None => {}
            }
            tracker.previous = cell;
        }
    }

    pub(super) fn finish(
        self,
    ) -> (
        EventMetrics,
        [MobilityMetrics; 4],
        usize,
        [IntentMetrics; 3],
    ) {
        let mobility = self.mobility.map(|mut tracker| {
            tracker.output.distinct_cells = tracker.visited.len();
            tracker.output
        });
        (
            self.events,
            mobility,
            self.initial_blocks - self.minimum_blocks,
            self.intents,
        )
    }
}

pub(super) fn admissions(cpus: &[CpuController]) -> [AdmissionMetrics; 3] {
    std::array::from_fn(|index| convert_admission(cpus[index].admission_trace_for_test()))
}

fn convert_admission(value: AdmissionTrace) -> AdmissionMetrics {
    AdmissionMetrics {
        expired_or_wrong_decision: value.expired_or_wrong_decision,
        missed_manual_tick: value.missed_manual_tick,
        stale_scene: value.stale_scene,
        invalid_hold: value.invalid_hold,
        contact: value.contact,
        strategic_plant: value.strategic_plant,
        invalid_shape: value.invalid_shape,
        pulse: value.pulse,
        route: value.route,
        duplicate_decision: value.duplicate_decision,
        refuge_handoff: value.refuge_handoff,
    }
}

pub(super) fn rng_draws(cpus: &[CpuController]) -> [u64; 3] {
    std::array::from_fn(|index| cpus[index].rng_draws())
}
