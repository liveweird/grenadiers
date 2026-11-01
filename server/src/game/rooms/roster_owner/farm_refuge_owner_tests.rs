use grenadiers_core::room::{CpuDifficulty, CpuProfile};

use super::farm_refuge_owner_support::run;

#[test]
fn current_blocked_birth_profiles_queue_plant_destroy_and_survive() {
    for (profile, difficulty) in [
        (
            CpuProfile::BLOCKED_BIRTHS_V42_STANDARD,
            CpuDifficulty::Standard,
        ),
        (CpuProfile::HARD_V43, CpuDifficulty::Hard),
        (
            CpuProfile::CONTINUATION_V44_STANDARD,
            CpuDifficulty::Standard,
        ),
        (CpuProfile::HARD_V45, CpuDifficulty::Hard),
    ] {
        let outcome = run(profile, difficulty);
        assert!(
            outcome.exercised_respawn_lairs,
            "{profile:?} delegated to the legacy no-lair planner"
        );
        assert!(outcome.planted, "{profile:?} did not plant at (3,8)");
        assert!(outcome.target_destroyed, "{profile:?} retained block (3,7)");
        assert!(
            outcome.survived,
            "{profile:?} did not survive its flame lifetime"
        );
        assert!(outcome.max_work <= grenadiers_core::cpu::survival::WORK_ALLOWANCE);
    }
}
