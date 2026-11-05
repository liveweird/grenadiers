use std::collections::BTreeSet;

use grenadiers_core::{
    cpu::WorldId,
    game::{CellId, State, StepEventKind, StepReport},
};

use super::super::cpu::{AdmissionTrace, CpuController};
use super::live_cpu_evaluation_types::{
    AdmissionMetrics, CadenceMetrics, EventMetrics, IntentMetrics, MobilityMetrics,
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
        collector.observe_positions(state, false);
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
        self.observe_positions(state, true);
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

    fn observe_positions(&mut self, state: &State, count_update: bool) {
        for (index, actor) in state.actors().iter().enumerate() {
            let cell = actor.position().cell();
            self.mobility[index].observe(cell, actor.alive(), count_update);
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

impl MobilityTracker {
    fn observe(&mut self, cell: Option<CellId>, alive: bool, count_update: bool) {
        if !alive {
            self.previous = None;
            self.stationary_span = 0;
            return;
        }
        if let Some(cell) = cell {
            self.visited.insert(cell);
        }
        if !count_update {
            self.previous = cell;
            return;
        }
        self.output.alive_updates += 1;
        match self.previous {
            Some(previous) if Some(previous) == cell => {
                self.output.stationary_updates += 1;
                self.stationary_span += 1;
                self.output.longest_stationary_span = self
                    .output
                    .longest_stationary_span
                    .max(self.stationary_span);
            }
            Some(_) => {
                self.output.cell_transitions += 1;
                self.stationary_span = 0;
            }
            None => {}
        }
        self.previous = cell;
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

pub(super) fn cadence_metrics(cpus: &[CpuController]) -> [CadenceMetrics; 3] {
    std::array::from_fn(|index| {
        let metrics = cpus[index].easy68_cadence_metrics_for_test();
        CadenceMetrics {
            pause_armed: metrics.pause_armed,
            pause_completed: metrics.pause_completed,
            planning_deferred: metrics.planning_deferred,
            planning_bypassed: metrics.planning_bypassed,
            guard_work: metrics.guard_work,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_updates_do_not_inflate_living_mobility_or_stationary_spans() {
        let first = CellId::new(1, 1);
        let second = CellId::new(2, 1);
        let third = CellId::new(3, 1);
        let mut eliminated = MobilityTracker::default();
        let mut survivor = MobilityTracker::default();
        eliminated.observe(Some(first), true, false);
        survivor.observe(Some(first), true, false);
        eliminated.observe(Some(first), true, true);
        eliminated.observe(Some(second), true, true);
        eliminated.observe(Some(second), true, true);
        survivor.observe(Some(second), true, true);
        let living = eliminated.output.clone();

        for _ in 0..4 {
            eliminated.observe(Some(second), false, true);
        }
        survivor.observe(Some(third), true, true);

        assert_eq!(eliminated.output, living);
        assert_eq!(eliminated.stationary_span, 0);
        assert_eq!(eliminated.output.alive_updates, 3);
        assert_eq!(eliminated.output.cell_transitions, 1);
        assert_eq!(eliminated.output.stationary_updates, 2);
        assert_eq!(eliminated.output.longest_stationary_span, 1);
        assert_eq!(survivor.output.alive_updates, 2);
        assert_eq!(survivor.output.cell_transitions, 2);
    }
}
