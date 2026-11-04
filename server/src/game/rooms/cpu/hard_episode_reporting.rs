use super::{outcome::Outcome, support::Case};
use crate::game::rooms::cpu::AdmissionTrace;

fn rejection_total(trace: &AdmissionTrace) -> u32 {
    trace.expired_or_wrong_decision
        + trace.missed_manual_tick
        + trace.stale_scene
        + trace.invalid_hold
        + trace.contact
        + trace.strategic_plant
        + trace.invalid_shape
        + trace.pulse
        + trace.route
        + trace.duplicate_decision
        + trace.refuge_handoff
}

fn admission_sum(admissions: &[AdmissionTrace; 3], value: impl Fn(&AdmissionTrace) -> u32) -> u32 {
    admissions.iter().map(value).sum()
}

pub(super) fn log(case: Case, outcome: &Outcome) {
    eprintln!(
        "{}",
        serde_json::json!({
            "case": case.label,
            "profile": outcome.profile.policy.version(),
            "constructionProfile": format!("{:?}", outcome.construction_profile),
            "generationVersion": outcome.generation_version,
            "pitLayout": format!("{:?}", outcome.pit_layout),
            "multiplier": outcome.multiplier.value(),
            "board": format!("{:?}", case.board),
            "lairs": case.lairs,
            "native": case.native,
            "moved": outcome.moved,
            "ownedBombs": outcome.owned_bombs,
            "eventPlacements": outcome.event_placements,
            "destroyedBlocks": outcome.destroyed_blocks,
            "flamesSeen": outcome.flames_seen,
            "mobBirths": outcome.mob_births,
            "mobMoved": outcome.mob_moved,
            "firstBombs": format!("{:?}", outcome.first_bombs),
            "upgrades": outcome.upgrades,
            "attackIntents": outcome.attack_intents,
            "attackBombs": outcome.attack_bombs,
            "completed": outcome.completed_switches,
            "alive": outcome.alive,
            "everDead": outcome.ever_dead,
            "dequeued": outcome.dequeued,
            "delivered": outcome.delivered,
            "cancelled": outcome.cancelled,
            "dropped": outcome.dropped,
            "maxWork": outcome.max_work,
            "admissionDenials": {
                "total": admission_sum(&outcome.admissions, rejection_total),
                "expired": admission_sum(&outcome.admissions, |trace| trace.expired_or_wrong_decision),
                "missedManual": admission_sum(&outcome.admissions, |trace| trace.missed_manual_tick),
                "stale": admission_sum(&outcome.admissions, |trace| trace.stale_scene),
                "contact": admission_sum(&outcome.admissions, |trace| trace.contact),
                "strategic": admission_sum(&outcome.admissions, |trace| trace.strategic_plant),
                "pulse": admission_sum(&outcome.admissions, |trace| trace.pulse),
                "route": admission_sum(&outcome.admissions, |trace| trace.route),
                "refugeHandoff": admission_sum(&outcome.admissions, |trace| trace.refuge_handoff),
            },
        })
    );
}

pub(super) fn assert_operational(outcome: &Outcome) {
    assert!(outcome.moved.into_iter().all(|value| value), "{outcome:?}");
    assert!(
        outcome.owned_bombs.into_iter().all(|value| value > 0),
        "{outcome:?}"
    );
    assert!(outcome.delivered > 0, "{outcome:?}");
    assert_eq!(
        outcome.dequeued,
        outcome.delivered + outcome.cancelled + outcome.failed + outcome.dropped,
        "{outcome:?}"
    );
    assert_eq!(outcome.dequeued, outcome.work_len, "{outcome:?}");
    assert_eq!(outcome.failed, 0, "{outcome:?}");
    assert!(
        outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
        "{outcome:?}"
    );
}
