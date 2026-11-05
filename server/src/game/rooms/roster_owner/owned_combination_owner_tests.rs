use grenadiers_core::{game::PickupKind, room::CpuProfile};

use super::owned_combination_owner_support::{ADDITIONAL_BLOCK, Outcome, run};

#[test]
fn queued_hard67_overlaps_one_useful_owned_bomb_after_a_real_capacity_pickup() {
    let hard66 = run(CpuProfile::HARD_V66);
    let hard67 = run(CpuProfile::HARD_V67);
    eprintln!("HARD67_OWNED_COMBINATION hard66={hard66:#?} hard67={hard67:#?}");

    assert_eq!(hard66.pickups, hard67.pickups);
    assert!(
        hard67
            .pickups
            .iter()
            .any(|(_, kind)| *kind == PickupKind::BombCapacity)
    );
    assert_baseline(&hard66);
    assert_eq!(hard67.plants.len(), 2, "{hard67:#?}");
    assert_eq!(hard66.plants[0], hard67.plants[0]);
    assert_ne!(hard67.plants[0].id, hard67.plants[1].id);
    assert!(hard67.plants[1].tick < hard67.plants[0].detonates_at);
    assert!(hard67.simultaneous_owned >= 2);
    assert!(hard67.additional_intact_at_second);
    assert!(second_reaches_additional_block(&hard67));
    assert_eq!(hard67.additional_destroyed_by, Some(hard67.plants[1].id));
    assert_eq!(
        hard67.additional_destroyed_at,
        Some(hard67.plants[1].detonates_at)
    );
    assert!(
        hard67
            .detonations
            .contains(&(hard67.plants[1].detonates_at, hard67.plants[1].id))
    );
    assert!(hard67.selected_rival_at_second);
    assert_eq!(
        hard67.route_profile_support_at_second,
        Some([false, false, true])
    );
    assert!(hard67.survived_both_windows);
    assert!(hard67.settled_progress_after_windows);
    assert_eq!(hard67.manual_attempts_after_first, 1, "{hard67:#?}");
    assert_accounted(&hard67);
    assert_eq!(hard67.admission, Default::default(), "{hard67:#?}");
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
