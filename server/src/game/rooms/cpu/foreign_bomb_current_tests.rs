use grenadiers_core::cpu::WorkBudget;
use grenadiers_core::cpu::routing::{PlantFailure, RouteFailure};
use grenadiers_core::cpu::survival::{
    ForeignBombRawHintMode, ForeignBombRawOutcome, HardPlantOutcome, PlantCandidateOutcome,
    TeleporterSupportFailure, WORK_ALLOWANCE, diagnose_foreign_bomb_raw_candidate,
    foreign_bomb_raw_hints,
};
use grenadiers_core::room::CpuPolicy;

use super::foreign_bomb_current_support::{Guard, probe, run_matrix};

#[test]
fn current_clearance_matrix_captures_real_foreign_bomb_farm_jobs_immutably() {
    let outcome = run_matrix();
    eprintln!(
        "FOREIGN_BOMB_CURRENT_MATRIX cases={} jobs={} captures={} at_approach={} guards={:?} effects={:?} living={:?} dead_owner={} equipment={:?} grace={} hazards=[pickups:{},flames:{},oil:{}] placements={} delivered={} failed={}",
        outcome.cases,
        outcome.jobs,
        outcome.captured_jobs,
        outcome.at_approach_jobs,
        outcome.guards,
        outcome.effects,
        outcome.living_actors,
        outcome.foreign_owner_dead,
        outcome.equipment,
        outcome.damage_grace,
        outcome.with_pickups,
        outcome.with_flames,
        outcome.with_oil,
        outcome.placements,
        outcome.delivered,
        outcome.failed,
    );
    assert_eq!(outcome.cases, 18);
    assert!(outcome.jobs > 0);
    assert!(outcome.placements > 0);
    assert_eq!(outcome.failed, 0);
    assert_eq!(outcome.delivered, outcome.jobs);
    assert_eq!(outcome.captured_jobs, 381);
    assert_eq!(outcome.at_approach_jobs, 194);
    assert_eq!(outcome.guards, [0, 0, 381, 0, 0]);
    assert_eq!(outcome.effects, [0, 0, 0]);
    assert_eq!(outcome.placements, 113);
    assert_eq!(outcome.examples.len(), 5);
    assert!(outcome.examples.iter().all(|capture| {
        capture.foreign_bombs > 0
            && capture.at_approach
            && capture.settled_stop <= capture.observation.parts().tick
            && capture.history.key() == capture.memory.key()
            && capture.rng.key() == capture.memory.key()
            && capture.actor != grenadiers_core::game::EntityId(u32::MAX)
            && !capture.foreign_deadlines.is_empty()
            && capture.guard != Guard::Other
            && capture
                .origin
                .as_ref()
                .is_none_or(|origin| origin.retained())
            && matches!(
                capture.policy,
                CpuPolicy::UtilityV62 | CpuPolicy::UtilityV64
            )
    }));
    for capture in &outcome.examples {
        eprintln!(
            "FOREIGN_BOMB_CURRENT_EXAMPLE case={:?} tick={:?} policy={:?} actor={:?} intent={:?} foreign={} deadlines={:?} guard={:?} effects={:?} living={} dead_owner={} equipment={:?} grace={} pickups={} flames={} oil={} origin={}",
            capture.case,
            capture.observation.parts().tick,
            capture.policy,
            capture.actor,
            capture.memory.strategic_intent(),
            capture.foreign_bombs,
            capture.foreign_deadlines,
            capture.guard,
            capture.effects,
            capture.living_actors,
            capture.foreign_owner_dead,
            capture.equipment,
            capture.damage_grace,
            capture.pickups,
            capture.flames,
            capture.oil,
            capture.origin.is_some(),
        );
    }
    let probes = outcome
        .examples
        .iter()
        .filter(|capture| capture.case.lairs && capture.origin.is_none())
        .map(probe)
        .collect::<Vec<_>>();
    eprintln!("FOREIGN_BOMB_CURRENT_PROBES {probes:#?}");
    assert_eq!(probes.len(), 2);
    assert!(
        probes
            .iter()
            .all(|probe| probe.rng_unchanged && probe.basis_unchanged)
    );
    let standard = probes
        .iter()
        .find(|probe| probe.case.difficulty == grenadiers_core::room::CpuDifficulty::Standard)
        .unwrap();
    assert_eq!(standard.outcome, HardPlantOutcome::CandidateRefused);
    assert_eq!(standard.usefulness, [Some(true); 5]);
    assert_eq!(standard.usefulness_spent, [105; 5]);
    assert!(standard.placements.iter().all(|placement| {
        placement
            .is_some_and(|placement| placement.detonates_at == grenadiers_core::game::Tick(766))
    }));
    assert!(standard.candidate_outcomes.iter().all(|outcome| {
        *outcome == PlantCandidateOutcome::TeleporterSupport(TeleporterSupportFailure::VisibleBombs)
    }));
    let standard_capture = outcome
        .examples
        .iter()
        .find(|capture| {
            capture.case.difficulty == grenadiers_core::room::CpuDifficulty::Standard
                && capture.case.lairs
        })
        .unwrap();
    assert_eq!(
        standard_capture.foreign_deadlines,
        [grenadiers_core::game::Tick(685)]
    );

    let mut raw = Vec::new();
    for capture in outcome.examples.iter().filter(|capture| capture.case.lairs) {
        let target = capture.memory.strategic_intent().unwrap().target;
        let candidate_allowance = probe(capture)
            .candidate_allowances
            .first()
            .copied()
            .unwrap_or(WORK_ALLOWANCE);
        let mut seen = Vec::new();
        for mode in [
            ForeignBombRawHintMode::Legacy,
            ForeignBombRawHintMode::BlockedAway,
        ] {
            let mut geometry = WorkBudget::new(WORK_ALLOWANCE);
            let hints = foreign_bomb_raw_hints(&capture.observation, target, mode, &mut geometry);
            eprintln!(
                "FOREIGN_BOMB_RAW_HINTS case={:?} mode={mode:?} spent={} hints={hints:?}",
                capture.case,
                geometry.spent(),
            );
            for hint in hints.into_iter().flatten() {
                if seen.contains(&hint) {
                    continue;
                }
                seen.push(hint);
                let diagnostic = diagnose_foreign_bomb_raw_candidate(
                    &capture.observation,
                    &capture.history,
                    capture.settled_stop,
                    target,
                    hint,
                    candidate_allowance,
                );
                eprintln!(
                    "FOREIGN_BOMB_RAW_RESULT case={:?} allowance={candidate_allowance} diagnostic={diagnostic:?}",
                    capture.case,
                );
                raw.push((capture.case, diagnostic));
            }
        }
    }
    assert_eq!(raw.len(), 4);
    assert!(raw.iter().all(|(_, diagnostic)| {
        diagnostic.total_spent <= diagnostic.allowance
            && diagnostic.total_spent
                == diagnostic.route_spent + diagnostic.usefulness_spent + diagnostic.contact_spent
    }));
    assert!(raw.iter().all(|(_, diagnostic)| {
        diagnostic.outcome
            == ForeignBombRawOutcome::Plant(PlantFailure::Route(RouteFailure::NoRoute))
    }));
    assert_eq!(
        raw.iter()
            .map(|(_, diagnostic)| diagnostic.total_spent)
            .collect::<Vec<_>>(),
        [1_063, 1_063, 1_067, 1_067]
    );
}

#[test]
#[ignore = "explicit public-basis evidence export; the asserted matrix test covers behavior"]
fn export_selected_current_public_basis_without_private_randomness() {
    let outcome = run_matrix();
    let capture = outcome
        .examples
        .iter()
        .find(|capture| {
            capture.case.difficulty == grenadiers_core::room::CpuDifficulty::Standard
                && capture.case.lairs
        })
        .expect("the selected current Standard+lairs basis must remain present");
    assert_eq!(
        capture.observation.parts().tick,
        grenadiers_core::game::Tick(579)
    );
    assert_eq!(capture.settled_stop, grenadiers_core::game::Tick(579));
    eprintln!(
        "FOREIGN_BOMB_CURRENT_PUBLIC_BASIS observation={:#?} memory={:#?} history={:#?} stop={:?} rng_key={:?} rng_draws={} origin={}",
        capture.observation,
        capture.memory,
        capture.history,
        capture.settled_stop,
        capture.rng.key(),
        capture.rng.draws(),
        capture.origin.is_some(),
    );
}
