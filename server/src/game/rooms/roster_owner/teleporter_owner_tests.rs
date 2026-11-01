use grenadiers_core::{
    cpu::survival::WORK_ALLOWANCE,
    room::{CpuDifficulty, CpuProfile},
};

use super::teleporter_owner_support::{Outcome, TARGET_BLOCK, run, run_native};

#[test]
fn standard_46_collects_then_plants_and_survives_with_lairs_off() {
    let outcome = run(
        CpuProfile::TELEPORTER_V46_STANDARD,
        CpuDifficulty::Standard,
        false,
    );
    eprintln!("TELEPORTER_OWNER {outcome:?}");
    assert_physical_outcome(&outcome);
    assert_eq!(outcome.named_target, TARGET_BLOCK);
    assert!(!outcome.positions.is_empty());
    assert!(!outcome.inputs.is_empty());
}

#[test]
fn inactive_profiles_complete_the_same_physical_witness_across_supported_modes() {
    for (profile, difficulty, lairs) in [
        (
            CpuProfile::TELEPORTER_V46_STANDARD,
            CpuDifficulty::Standard,
            true,
        ),
        (CpuProfile::HARD_V47, CpuDifficulty::Hard, false),
        (CpuProfile::TELEPORTER_V46_EASY, CpuDifficulty::Easy, false),
    ] {
        let outcome = run(profile, difficulty, lairs);
        eprintln!("TELEPORTER_OWNER_VARIANT profile={profile:?} lairs={lairs} outcome={outcome:?}");
        assert_physical_outcome(&outcome);
        assert_eq!(
            outcome.named_target,
            if difficulty == CpuDifficulty::Hard {
                grenadiers_core::game::CellId::new(2, 7)
            } else {
                TARGET_BLOCK
            }
        );
    }
}

#[test]
fn native_worker_completes_collection_stop_and_productive_follow_up() {
    let outcome = run_native(CpuProfile::TELEPORTER_V50_STANDARD, CpuDifficulty::Standard);
    eprintln!("TELEPORTER_OWNER_NATIVE {outcome:?}");
    assert_physical_outcome(&outcome);
    assert_eq!(outcome.named_target, TARGET_BLOCK);
}

#[test]
fn current_profiles_complete_physical_handoff_and_useful_follow_up() {
    for (profile, difficulty, lairs) in [
        (
            CpuProfile::TELEPORTER_V50_STANDARD,
            CpuDifficulty::Standard,
            false,
        ),
        (
            CpuProfile::TELEPORTER_V50_STANDARD,
            CpuDifficulty::Standard,
            true,
        ),
        (CpuProfile::TELEPORTER_V50_EASY, CpuDifficulty::Easy, false),
        (CpuProfile::HARD_V51, CpuDifficulty::Hard, false),
    ] {
        assert_physical_outcome(&run(profile, difficulty, lairs));
    }
}

fn assert_physical_outcome(outcome: &Outcome) {
    assert!(outcome.movement_admission_ticks.len() >= 2);
    assert_eq!(
        outcome.movement_update_ticks.len(),
        outcome.movement_admission_ticks.len()
    );
    assert_eq!(
        outcome.movement_sealed_ticks,
        outcome.movement_admission_ticks
    );
    assert_eq!(
        outcome.stop_submission_ticks.len(),
        outcome.movement_admission_ticks.len()
    );
    assert_eq!(outcome.stop_sealed_ticks, outcome.stop_submission_ticks);
    assert_eq!(
        outcome.physical_stop_ticks.len(),
        outcome.movement_admission_ticks.len()
    );
    for index in 0..outcome.movement_admission_ticks.len() {
        let admission = outcome.movement_admission_ticks[index];
        assert_eq!(outcome.movement_update_ticks[index].0, admission.0 + 1);
        assert_eq!(outcome.stop_submission_ticks[index].0, admission.0 + 1);
        assert_eq!(outcome.physical_stop_ticks[index].0, admission.0 + 2);
    }
    assert!(outcome.stop_pose_held.iter().all(|held| *held));
    assert!(outcome.stop_inputs_cleared.iter().all(|clear| *clear));
    assert_eq!(outcome.cpu_teleport_tick, outcome.collection_tick);
    assert_eq!(outcome.human_teleport_tick, outcome.collection_tick);
    assert!(outcome.cpu_landed_at_human_pose);
    assert!(outcome.human_landed_at_collection_endpoint);
    assert!(outcome.stopped_completion_tick > outcome.collection_tick);
    assert!(outcome.plant_tick > outcome.stopped_completion_tick);
    assert!(outcome.plant_cell.is_some());
    assert!(outcome.named_target_existed);
    assert!(outcome.plant_reaches_named_target);
    assert!(outcome.target_destroyed && outcome.survived_flames);
    assert_eq!(outcome.failed_work, 0);
    assert!(outcome.max_work <= WORK_ALLOWANCE);
}
