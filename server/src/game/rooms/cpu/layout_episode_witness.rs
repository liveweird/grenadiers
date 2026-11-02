use grenadiers_core::{
    cpu::{
        CpuRng, MovementIntent, Observation, WorkBudget,
        forecast::{ForecastLimits, ObservedHazardMemory, TickWindow, forecast},
        policy::UtilityMemory,
        routing::{PulseRequest, RouteMode, certify_lair_pulse},
        survival::{
            HardPlantDiagnostic, MAX_PLAN_AGE_TICKS, OriginRecoveryChoice, PlantEscapeOrigin,
            SurvivalPlan, plan_continuation_lairs_v62, plan_hard_contact_punch_v61_with_origin,
        },
    },
    game::{
        CellId, Direction, EffectKind, EntityId, SeatInput, State, StepEventKind, StepReport, Tick,
    },
    room::CpuPolicy,
};

use crate::game::rooms::cpu::{PlannedResultWitness, PumpReport};

use super::{BombLifecycle, DeathWitness, EFFECTS};

#[path = "layout_plant_diagnostic.rs"]
mod plant_diagnostic;
use plant_diagnostic::FullProof;
pub(super) use plant_diagnostic::{demolition_enabled, digest};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms::roster_owner) struct PlanSnapshot {
    pub policy: CpuPolicy,
    pub observation: Observation,
    pub memory: UtilityMemory,
    pub hazard_memory: ObservedHazardMemory,
    pub settled_stop_tick: Tick,
    pub rng_key: grenadiers_core::cpu::MemoryKey,
    pub rng_draws_before: u64,
    pub rng_draws_after: u64,
    pub escape_origin: Option<EscapeOriginWitness>,
    pub plan: Option<SurvivalPlan>,
    pub has_contact: bool,
    pub work_spent: u32,
    pub full_proof: FullProof,
    pub planner_prefix_spent: u32,
    pub plant_diagnostic: Option<HardPlantDiagnostic>,
    pub corrected_recovery: Option<CorrectedRecoveryProof>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms::roster_owner) struct StationaryPreflight {
    pub publication: Tick,
    pub safe_until: Tick,
    pub hold_ticks: u8,
    pub spent: u32,
    pub outcome: Result<(), &'static str>,
}

pub(super) struct TraceIdentity {
    pub instance: u64,
    pub id: &'static str,
    pub code: &'static str,
}

pub(super) const fn trace_identity(matrix: bool) -> TraceIdentity {
    if matrix {
        TraceIdentity {
            instance: 4_028,
            id: "0123456789abcdef0123456789abcdef",
            code: "ABCDEFGH",
        }
    } else {
        TraceIdentity {
            instance: 4_029,
            id: "1123456789abcdef0123456789abcdef",
            code: "BCDEFGHJ",
        }
    }
}

#[derive(Clone)]
pub(in crate::game::rooms::roster_owner) struct EscapeOriginWitness(PlantEscapeOrigin);

impl std::fmt::Debug for EscapeOriginWitness {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PlantEscapeOrigin(..)")
    }
}

impl PartialEq for EscapeOriginWitness {
    fn eq(&self, other: &Self) -> bool {
        self.0.same_origin(&other.0)
    }
}

impl Eq for EscapeOriginWitness {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms::roster_owner) struct CorrectedRecoveryProof {
    pub spent: u32,
    pub outcome: CorrectedRecoveryOutcome,
    pub legs: Vec<(MovementIntent, u8)>,
    pub positions: Vec<(Tick, grenadiers_core::game::Position, Direction)>,
    pub survive_until: Option<Tick>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms::roster_owner) enum CorrectedRecoveryOutcome {
    Continuation,
    Pulse,
    None,
}

impl PlanSnapshot {
    pub(super) fn capture(witness: PlannedResultWitness<'_>, demolition_enabled: bool) -> Self {
        let mut full_work = WorkBudget::new(65_536);
        let mut full_rng: CpuRng = witness.basis.rng.clone();
        let full_result = plan_hard_contact_punch_v61_with_origin(
            witness.basis.observation,
            witness.basis.memory,
            witness.basis.hazard_memory,
            witness.basis.settled_stop_tick,
            witness.basis.escape_origin,
            &mut full_rng,
            &mut full_work,
        );
        let full_proof = match full_result {
            Ok(plan) => FullProof {
                spent: full_work.spent(),
                plan: Some(plan.survival),
                error: None,
            },
            Err(error) => FullProof {
                spent: full_work.spent(),
                plan: None,
                error: Some(error),
            },
        };
        let (planner_prefix_spent, plant_diagnostic) =
            plant_diagnostic::capture(&witness, full_work.spent(), demolition_enabled);
        let corrected_recovery = witness.basis.escape_origin.map(|origin| {
            let mut corrected_work = WorkBudget::new(65_536);
            let (outcome, legs, positions, survive_until) = match plan_continuation_lairs_v62(
                witness.basis.observation,
                witness.basis.hazard_memory,
                witness.basis.settled_stop_tick,
                witness.basis.memory,
                origin,
                &mut corrected_work,
            ) {
                Some(OriginRecoveryChoice::Continuation(certificate)) => (
                    CorrectedRecoveryOutcome::Continuation,
                    certificate.diagnostic_legs(),
                    certificate.diagnostic_positions(),
                    Some(certificate.diagnostic_survive_until()),
                ),
                Some(OriginRecoveryChoice::Pulse(_)) => (
                    CorrectedRecoveryOutcome::Pulse,
                    Vec::new(),
                    Vec::new(),
                    None,
                ),
                None => (CorrectedRecoveryOutcome::None, Vec::new(), Vec::new(), None),
            };
            CorrectedRecoveryProof {
                spent: corrected_work.spent(),
                outcome,
                legs,
                positions,
                survive_until,
            }
        });
        Self {
            policy: *witness.basis.policy,
            observation: witness.basis.observation.clone(),
            memory: *witness.basis.memory,
            hazard_memory: witness.basis.hazard_memory.clone(),
            settled_stop_tick: witness.basis.settled_stop_tick,
            rng_key: witness.basis.rng.key(),
            rng_draws_before: witness.rng_draws_before,
            rng_draws_after: witness.rng_draws_after,
            escape_origin: witness
                .basis
                .escape_origin
                .cloned()
                .map(EscapeOriginWitness),
            plan: witness.plan.cloned(),
            has_contact: witness.has_contact,
            work_spent: witness.work_spent,
            full_proof,
            planner_prefix_spent,
            plant_diagnostic,
            corrected_recovery,
        }
    }
}

pub(in crate::game::rooms::roster_owner) fn pulse_refusal_probes(
    basis: &PlanSnapshot,
) -> Vec<(RouteMode, MovementIntent, u8, u32, u32, String)> {
    let mut probes = Vec::new();
    for mode in [
        RouteMode::CertifiedOnly,
        RouteMode::AllowStationaryAssumptions,
    ] {
        for (movement, hold_ticks) in [
            (MovementIntent::Stop, 1),
            (MovementIntent::Direction(Direction::North), 3),
            (MovementIntent::Direction(Direction::North), 12),
            (MovementIntent::Direction(Direction::East), 3),
            (MovementIntent::Direction(Direction::East), 12),
            (MovementIntent::Direction(Direction::South), 3),
            (MovementIntent::Direction(Direction::South), 12),
            (MovementIntent::Direction(Direction::West), 3),
            (MovementIntent::Direction(Direction::West), 12),
        ] {
            let mut work = WorkBudget::new(65_536);
            let hazards = forecast(
                &basis.observation,
                &basis.hazard_memory,
                None,
                ForecastLimits::default(),
                &mut work,
            )
            .unwrap();
            let forecast_spent = work.spent();
            let tick = basis.observation.parts().tick;
            let start_window = TickWindow {
                start: Tick(tick.0 + 1),
                end: Tick(tick.0 + MAX_PLAN_AGE_TICKS + 2),
            };
            let request = PulseRequest {
                movement,
                hold_ticks,
                start_window,
                stopped_safe_until: Tick(start_window.end.0 + u64::from(hold_ticks) + 9),
                mode,
                settled_stop_tick: basis.settled_stop_tick,
            };
            let outcome = match certify_lair_pulse(&basis.observation, &hazards, request, &mut work)
            {
                Ok(_) => "ok-before-pit-guard".to_owned(),
                Err(error) => format!("{error:?}"),
            };
            probes.push((
                mode,
                movement,
                hold_ticks,
                forecast_spent,
                work.spent(),
                outcome,
            ));
        }
    }
    probes
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms::roster_owner) struct StepReplayWitness {
    pub before: State,
    pub inputs: Vec<SeatInput>,
    pub report: StepReport,
    pub after: State,
}

impl StepReplayWitness {
    pub(super) fn prove(
        before: State,
        inputs: &[SeatInput],
        report: &StepReport,
        after: &State,
    ) -> Self {
        let mut replay = before.clone();
        let replayed = replay.step(inputs).unwrap();
        assert_eq!(replayed, *report, "tick report must replay exactly");
        assert_eq!(replay, *after, "post-step state must replay exactly");
        Self {
            before,
            inputs: inputs.to_vec(),
            report: report.clone(),
            after: after.clone(),
        }
    }
}

pub(super) fn merge(total: &mut PumpReport, next: PumpReport) {
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

pub(super) fn record_events(
    report: &StepReport,
    actor_ids: &[EntityId; 3],
    counts: &mut [usize; 3],
    first: &mut [Option<BombLifecycle>; 3],
) {
    for event in &report.events {
        if let StepEventKind::BombPlaced(bomb) = event.kind
            && let Some(index) = actor_ids.iter().position(|id| *id == bomb.owner)
        {
            counts[index] += 1;
            first[index].get_or_insert(BombLifecycle {
                id: bomb.id,
                owner: bomb.owner,
                cell: bomb.cell,
                trigger: bomb.trigger,
                placed_at: event.tick,
                detonated_at: None,
                all_finite_flames_clear_at: None,
            });
        }
    }
}

pub(super) fn update_lifecycles(
    report: &StepReport,
    state: &State,
    first: &mut [Option<BombLifecycle>; 3],
) {
    for event in &report.events {
        if let StepEventKind::BombDetonated(bomb) = event.kind {
            for lifecycle in first.iter_mut().flatten() {
                if lifecycle.id == bomb.id {
                    lifecycle.detonated_at = Some(event.tick);
                }
            }
        }
    }
    if state.public_view().flames.is_empty() {
        for lifecycle in first.iter_mut().flatten() {
            if lifecycle.detonated_at.is_some() && lifecycle.all_finite_flames_clear_at.is_none() {
                lifecycle.all_finite_flames_clear_at = Some(state.tick());
            }
        }
    }
}

pub(super) fn death_witness(
    state: &State,
    phase: grenadiers_core::room::MatchPhase,
    report: &StepReport,
) -> Option<DeathWitness> {
    let eliminated = report
        .events
        .iter()
        .filter_map(|event| match event.kind {
            StepEventKind::ActorEliminated { seat, .. } => Some(seat),
            _ => None,
        })
        .collect::<Vec<_>>();
    if eliminated.is_empty() {
        return None;
    }
    Some(DeathWitness {
        tick: report.tick,
        phase,
        eliminated,
        positions: state
            .actors()
            .iter()
            .map(|actor| actor.position())
            .collect(),
        effects: state
            .actors()
            .iter()
            .map(|actor| {
                EFFECTS
                    .into_iter()
                    .filter(|effect| actor.effects().active(*effect, state.tick()))
                    .collect::<Vec<EffectKind>>()
            })
            .collect(),
        pits: state.spike_pits(),
        mobs: state.mobs(),
        fire: state
            .public_view()
            .flames
            .into_iter()
            .map(|flame| flame.cell)
            .collect::<Vec<CellId>>(),
        report: report.clone(),
    })
}
