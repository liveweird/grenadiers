use grenadiers_core::{
    cpu::policy::{StrategicObjectiveKind, StrategicPhase},
    cpu::{ObservedActor, survival::PlantCandidateOutcome},
    game::{ArenaConstructionProfile, CellId, SeatId, StepEventKind, Tick},
    room::{BoardSize, SpikePitSettings},
};

use super::trace::{LayoutCase, run_zero_bomber_trace};

#[test]
fn hard_v63_zero_bomber_reports_first_attack_plant_refusal_and_death() {
    let result = run_zero_bomber_trace(
        LayoutCase {
            label: "zero-bomber-seat-two",
            board: BoardSize::ThirteenByEleven,
            lairs: true,
            seed: 73,
            profile: ArenaConstructionProfile::LairClearanceV5,
            pit_settings: SpikePitSettings::new(60, 300).unwrap(),
        },
        840,
        SeatId::Two,
    );
    assert_eq!(result.event_placements, [0, 2, 2]);
    let alive = [SeatId::Two, SeatId::Three, SeatId::Four]
        .map(|seat| result.frames.last().unwrap().actors[seat.index()].alive);
    assert_eq!(alive, [false, false, true]);
    let attack = result
        .plan_witnesses
        .iter()
        .find(|witness| reached_attack_approach(witness))
        .expect("Seat Two physically reaches an Attack approach");
    let refusal = result
        .plan_witnesses
        .iter()
        .find(|witness| {
            witness.observation.parts().tick >= attack.observation.parts().tick
                && reached_attack_approach(witness)
                && witness
                    .plant_diagnostic
                    .as_ref()
                    .is_some_and(|diagnostic| !diagnostic.candidates.is_empty())
        })
        .expect("first actual Attack plant candidate attempt");
    let attempts = result
        .plan_witnesses
        .iter()
        .filter(|witness| {
            witness.observation.parts().tick >= refusal.observation.parts().tick
                && witness
                    .plant_diagnostic
                    .as_ref()
                    .is_some_and(|diagnostic| !diagnostic.candidates.is_empty())
        })
        .take(8)
        .map(|witness| {
            (
                witness.observation.parts().tick,
                recipient_position(witness),
                witness
                    .plan
                    .as_ref()
                    .and_then(|plan| plan.control.next_memory.strategic_intent()),
                witness.planner_prefix_spent,
                witness.plant_diagnostic.clone().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let all_attempts = result
        .plan_witnesses
        .iter()
        .filter(|witness| {
            witness
                .plant_diagnostic
                .as_ref()
                .is_some_and(|diagnostic| !diagnostic.candidates.is_empty())
        })
        .collect::<Vec<_>>();
    let replay_mismatches = all_attempts
        .iter()
        .filter_map(|witness| {
            let diagnostic = witness.plant_diagnostic.as_ref().unwrap();
            (witness.planner_prefix_spent + diagnostic.total_spent != witness.work_spent).then_some(
                (
                    witness.observation.parts().tick,
                    witness.planner_prefix_spent,
                    diagnostic.total_spent,
                    witness.work_spent,
                    diagnostic.outcome,
                ),
            )
        })
        .collect::<Vec<_>>();
    assert!(replay_mismatches.is_empty(), "{replay_mismatches:?}");
    let counterfactuals = all_attempts
        .iter()
        .filter_map(|witness| {
            witness
                .plant_diagnostic
                .as_ref()?
                .demolition
                .as_ref()
                .map(|proof| (*witness, proof))
        })
        .collect::<Vec<_>>();
    assert_eq!(counterfactuals.len(), 1);
    let (counter_basis, counterfactual) = counterfactuals[0];
    assert_eq!(counter_basis.observation.parts().tick, Tick(183));
    assert_eq!(counter_basis.memory.strategic_intent(), None);
    assert_eq!(counterfactual.block, CellId::new(11, 3));
    assert_eq!(counterfactual.visible_yield, 1);
    assert_eq!(counterfactual.useful, Some(true));
    assert_eq!(counterfactual.contact, Ok(()));
    assert_eq!(counterfactual.survive_until, Some(Tick(400)));
    assert!(
        counter_basis.planner_prefix_spent + counterfactual.total_spent
            <= grenadiers_core::cpu::survival::WORK_ALLOWANCE
    );
    let candidate_timeline = all_attempts
        .iter()
        .map(|witness| {
            let plan = witness.plan.as_ref().unwrap();
            (
                witness.observation.parts().tick,
                recipient_position(witness),
                plan.control.next_memory.strategic_intent(),
                witness
                    .plant_diagnostic
                    .as_ref()
                    .unwrap()
                    .candidates
                    .clone(),
            )
        })
        .collect::<Vec<_>>();
    let supported_useful_jobs = all_attempts
        .iter()
        .filter_map(|witness| {
            witness
                .plant_diagnostic
                .as_ref()?
                .candidates
                .iter()
                .any(|candidate| {
                    candidate.derived_exact_usefulness == Some(true)
                        && !matches!(
                            candidate.outcome,
                            PlantCandidateOutcome::TeleporterSupport(_)
                        )
                })
                .then_some(witness.observation.parts().tick)
        })
        .collect::<Vec<_>>();
    let last_attempt = all_attempts.last().map(|witness| {
        (
            witness.observation.parts().tick,
            recipient_position(witness),
            witness.plan.as_ref().map(|plan| plan.control),
            witness.plant_diagnostic.as_ref(),
        )
    });
    let first_attempt = all_attempts.first().map(|witness| {
        (
            witness.observation.parts().tick,
            recipient_position(witness),
            witness.plan.as_ref().map(|plan| plan.control),
            witness.plant_diagnostic.as_ref(),
        )
    });
    let first_past_teleporter = all_attempts.iter().find_map(|witness| {
        let diagnostic = witness.plant_diagnostic.as_ref()?;
        diagnostic
            .candidates
            .iter()
            .any(|candidate| {
                !matches!(
                    candidate.outcome,
                    PlantCandidateOutcome::TeleporterSupport(_)
                )
            })
            .then_some((
                witness.observation.parts().tick,
                recipient_position(witness),
                witness.plan.as_ref().map(|plan| plan.control),
                diagnostic,
            ))
    });
    let first_past_teleporter_after_clear = all_attempts.iter().find_map(|witness| {
        if witness.observation.parts().tick < Tick(541) {
            return None;
        }
        let diagnostic = witness.plant_diagnostic.as_ref()?;
        diagnostic
            .candidates
            .iter()
            .any(|candidate| {
                !matches!(
                    candidate.outcome,
                    PlantCandidateOutcome::TeleporterSupport(_)
                )
            })
            .then_some((
                witness.observation.parts().tick,
                recipient_position(witness),
                witness.plan.as_ref().map(|plan| plan.control),
                diagnostic,
            ))
    });
    let death = result
        .frames
        .iter()
        .find(|frame| {
            frame.report.events.iter().any(|event| {
                matches!(
                    event.kind,
                    StepEventKind::ActorEliminated {
                        seat: SeatId::Two,
                        ..
                    }
                )
            })
        })
        .expect("Seat Two death");
    let detonations = death
        .report
        .events
        .iter()
        .filter_map(|event| match event.kind {
            StepEventKind::BombDetonated(bomb) => Some(bomb),
            _ => None,
        })
        .collect::<Vec<_>>();
    let start = Tick(refusal.observation.parts().tick.0.saturating_sub(6));
    let end = Tick(refusal.observation.parts().tick.0 + 12);
    let movement = result
        .frames
        .iter()
        .filter(|frame| (start..=end).contains(&frame.tick))
        .map(|frame| {
            let actor = frame
                .actors
                .iter()
                .find(|actor| actor.seat == SeatId::Two)
                .unwrap();
            (frame.tick, actor.position, frame.accepted_inputs.clone())
        })
        .collect::<Vec<_>>();
    let plan = refusal.plan.as_ref().unwrap();
    let actor = recipient_position(refusal).unwrap();
    eprintln!(
        "setup=profile63/13x11/V5/lairs/seed73/pits60-300/room4028 \
        initial_world={} initial_actors={:?} initial_pits={:?} initial_cloud={:?} \
        placements={:?} alive={alive:?}\n\
        attack_tick={:?} actor={actor:?} intent={:?} control={:?} \
        work={} full_work={} prefix={} diagnostic={:?} attempts={attempts:#?} \
        attempt_count={} first_attempt={first_attempt:?} \
        supported_useful_jobs={supported_useful_jobs:?} \
        candidate_timeline={candidate_timeline:#?} \
        first_past_teleporter={first_past_teleporter:?} \
        first_past_teleporter_after_clear={first_past_teleporter_after_clear:?} \
        last_attempt={last_attempt:?} \
        movement={movement:#?}\n\
        death_tick={:?} detonations={detonations:#?} death_report={:#?}",
        digest(&result.initial_world),
        result.initial_actors,
        result.initial_pits,
        result.initial_cloud,
        result.event_placements,
        refusal.observation.parts().tick,
        plan.control.next_memory.strategic_intent(),
        plan.control,
        refusal.work_spent,
        refusal.full_proof.spent,
        refusal.planner_prefix_spent,
        refusal.plant_diagnostic,
        all_attempts.len(),
        death.tick,
        death.report
    );
    assert!(
        refusal
            .plan
            .as_ref()
            .unwrap()
            .execution
            .as_ref()
            .and_then(|execution| execution.plant())
            .is_none()
    );
}

fn digest(value: &impl std::fmt::Debug) -> u64 {
    format!("{value:?}")
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3)
        })
}

fn reached_attack_approach(witness: &super::trace::witness::PlanSnapshot) -> bool {
    let Some(position) = recipient_position(witness) else {
        return false;
    };
    witness.plan.as_ref().is_some_and(|plan| {
        plan.control
            .next_memory
            .strategic_intent()
            .is_some_and(|intent| {
                intent.objective == StrategicObjectiveKind::Attack
                    && matches!(
                        intent.phase,
                        StrategicPhase::Approach | StrategicPhase::Reengage
                    )
                    && position.cell() == intent.approach_cell
            })
    })
}

fn recipient_position(
    witness: &super::trace::witness::PlanSnapshot,
) -> Option<grenadiers_core::game::Position> {
    witness
        .observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            ObservedActor::Visible(actor)
                if actor.seat == witness.observation.parts().recipient =>
            {
                Some(actor.position)
            }
            _ => None,
        })
}
