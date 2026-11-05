use grenadiers_core::room::CpuProfile;

use super::easy68_cadence_owner_support::{Outcome, run, run_native};

#[test]
fn queued_generated_easy68_completes_one_certified_pause_and_resumes() {
    let standard = run(CpuProfile::RECOVERY_V62_STANDARD);
    let easy = run(CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    eprintln!("EASY68_CERTIFIED_CADENCE standard={standard:#?} easy={easy:#?}");

    assert_eq!(standard.profile, CpuProfile::RECOVERY_V62_STANDARD);
    assert!(standard.moved, "{standard:#?}");
    assert_eq!(standard.metrics, Default::default(), "{standard:#?}");
    assert_eq!(easy.profile, CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    assert!(easy.moved, "{easy:#?}");
    assert!(easy.metrics.pause_armed > 0, "{easy:#?}");
    assert!(easy.metrics.pause_completed > 0, "{easy:#?}");
    assert!(easy.metrics.planning_deferred >= 12, "{easy:#?}");
    assert!(easy.metrics.guard_work > 0, "{easy:#?}");
    assert!(easy.confirmed_stop_at.is_some(), "{easy:#?}");
    assert!(easy.resumed_after_pause_at.is_some(), "{easy:#?}");
    assert_accounted(&standard);
    assert_accounted(&easy);
}

#[test]
fn native_easy68_workers_complete_pause_resume_bomb_progress_and_danger_bypass() {
    let outcome = run_native();
    eprintln!("EASY68_CERTIFIED_CADENCE_NATIVE {outcome:#?}");

    assert_eq!(outcome.profile, CpuProfile::CERTIFIED_CADENCE_V68_EASY);
    assert!(outcome.moved, "{outcome:#?}");
    assert!(outcome.owned_bomb_seen, "{outcome:#?}");
    assert!(outcome.metrics.pause_completed > 0, "{outcome:#?}");
    assert!(outcome.metrics.planning_deferred >= 12, "{outcome:#?}");
    assert!(outcome.metrics.planning_bypassed > 0, "{outcome:#?}");
    assert!(outcome.first_bypass_at.is_some(), "{outcome:#?}");
    assert!(outcome.confirmed_stop_at.is_some(), "{outcome:#?}");
    assert!(outcome.resumed_after_pause_at.is_some(), "{outcome:#?}");
    assert_accounted(&outcome);
}

fn assert_accounted(outcome: &Outcome) {
    assert!(outcome.delivered > 0, "{outcome:#?}");
    assert_eq!((outcome.failed, outcome.dropped), (0, 0), "{outcome:#?}");
    assert!(outcome.cancelled <= 1, "{outcome:#?}");
    assert_eq!(
        outcome.dequeued,
        outcome.delivered + outcome.failed + outcome.cancelled + outcome.dropped,
        "{outcome:#?}"
    );
    assert!(outcome.queue_drained, "{outcome:#?}");
    assert!(
        outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE,
        "{outcome:#?}"
    );
}
