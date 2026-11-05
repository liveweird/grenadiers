use grenadiers_core::{game::PickupKind, room::CpuProfile};

use super::owned_combination_owner_support::{ADDITIONAL_BLOCK, Outcome, run};

#[test]
fn queued_hard67_overlaps_one_useful_owned_bomb_after_a_real_capacity_pickup() {
    let hard66 = run(CpuProfile::HARD_V66);
    let hard67 = run(CpuProfile::HARD_V67);
    eprintln!("HARD67_OWNED_COMBINATION hard66={hard66:#?} hard67={hard67:#?}");

    assert_positive(&hard66, &hard67);
}

#[test]
fn queued_hard69_reuses_the_exact_physical_combination_proof() {
    let hard66 = run(CpuProfile::HARD_V66);
    let hard69 = run(CpuProfile::HARD_V69);
    eprintln!("HARD69_OWNED_COMBINATION hard66={hard66:#?} hard69={hard69:#?}");

    assert_positive(&hard66, &hard69);
}

fn assert_positive(hard66: &Outcome, current: &Outcome) {
    assert_eq!(hard66.pickups, current.pickups);
    assert!(
        current
            .pickups
            .iter()
            .any(|(_, kind)| *kind == PickupKind::BombCapacity)
    );
    assert_baseline(hard66);
    assert_eq!(current.plants.len(), 2, "{current:#?}");
    assert_eq!(hard66.plants[0], current.plants[0]);
    assert_ne!(current.plants[0].id, current.plants[1].id);
    assert!(current.plants[1].tick < current.plants[0].detonates_at);
    assert!(current.simultaneous_owned >= 2);
    assert!(current.additional_intact_at_second);
    assert!(second_reaches_additional_block(current));
    assert_eq!(current.additional_destroyed_by, Some(current.plants[1].id));
    assert_eq!(
        current.additional_destroyed_at,
        Some(current.plants[1].detonates_at)
    );
    assert!(
        current
            .detonations
            .contains(&(current.plants[1].detonates_at, current.plants[1].id))
    );
    assert!(current.selected_rival_at_second);
    assert_eq!(
        current.route_profile_support_at_second,
        Some([false, false, true, true])
    );
    assert!(current.survived_both_windows);
    assert!(current.settled_progress_after_windows);
    assert_eq!(current.manual_attempts_after_first, 1, "{current:#?}");
    assert_accounted(current);
    assert_eq!(current.admission, Default::default(), "{current:#?}");
}

fn assert_baseline(outcome: &Outcome) {
    assert_eq!(outcome.profile, CpuProfile::HARD_V66);
    assert!(outcome.plants.len() >= 2, "{outcome:#?}");
    assert!(outcome.plants[1].tick >= outcome.plants[0].detonates_at);
    assert_eq!(outcome.simultaneous_owned, 1);
    assert!(outcome.additional_destroyed_at.is_none());
    assert!(
        outcome
            .detonations
            .contains(&(outcome.plants[0].detonates_at, outcome.plants[0].id))
    );
    assert_eq!(outcome.manual_attempts_after_first, 1, "{outcome:#?}");
    assert_eq!(outcome.admission.refuge_handoff, 2, "{outcome:#?}");
    let mut non_handoff = outcome.admission.clone();
    non_handoff.refuge_handoff = 0;
    assert_eq!(non_handoff, Default::default(), "{outcome:#?}");
    assert_accounted(outcome);
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

fn second_reaches_additional_block(outcome: &Outcome) -> bool {
    let second = outcome.plants[1];
    if second.blast != grenadiers_core::game::BlastKind::Standard {
        return false;
    }
    let column = second.cell.column.abs_diff(ADDITIONAL_BLOCK.column);
    let row = second.cell.row.abs_diff(ADDITIONAL_BLOCK.row);
    (column == 0 && row <= second.range) || (row == 0 && column <= second.range)
}
