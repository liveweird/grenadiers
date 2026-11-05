use crate::cpu::WorkBudget;
use crate::game::{CellId, PickupKind, SeatId};

use super::{Candidate, OpeningMode, StrategicObjectiveKind, StrategicTarget, promote_fresh};

fn candidate(
    objective: StrategicObjectiveKind,
    target: StrategicTarget,
    score: u8,
    distance: u16,
) -> Candidate {
    Candidate {
        objective,
        target,
        approach: CellId::new(1, 1),
        score,
        distance,
        first_step: CellId::new(1, 1),
    }
}

fn attack(score: u8, distance: u16) -> Candidate {
    candidate(
        StrategicObjectiveKind::Attack,
        StrategicTarget::Opponent { seat: SeatId::One },
        score,
        distance,
    )
}

fn farm(cell: CellId, distance: u16) -> Candidate {
    candidate(
        StrategicObjectiveKind::Farm,
        StrategicTarget::Block { cell },
        2,
        distance,
    )
}

fn choose(
    baseline: Candidate,
    candidates: &[Candidate],
    excluded: Option<StrategicTarget>,
    allowance: u32,
) -> (Candidate, u32) {
    let mut work = WorkBudget::new(allowance);
    let chosen = promote_fresh(
        OpeningMode::FarmOpportunity,
        baseline,
        candidates,
        excluded,
        &mut work,
    );
    (chosen, work.spent())
}

#[test]
fn far_score_four_attack_yields_to_first_near_farm_only_on_strict_ratio() {
    let baseline = attack(4, 16);
    let near = farm(CellId::new(3, 3), 2);
    let farther = farm(CellId::new(5, 5), 3);
    let candidates = [baseline, near, farther];
    let (chosen, spent) = choose(baseline, &candidates, None, 64);
    assert_eq!(chosen.target, near.target);
    assert_eq!(spent, 3);

    for (attack_distance, farm_distance) in [(0, 2), (1, 2), (5, 2)] {
        let baseline = attack(4, attack_distance);
        let farm = farm(CellId::new(3, 3), farm_distance);
        assert_eq!(
            choose(baseline, &[baseline, farm], None, 64).0.target,
            baseline.target
        );
    }

    let weak = attack(1, 30);
    let near = farm(CellId::new(3, 3), 1);
    assert_eq!(choose(weak, &[weak, near], None, 64).0.target, weak.target);
}

#[test]
fn any_eligible_collect_vetoes_promotion_even_when_it_loses_sort_order() {
    let baseline = attack(4, 16);
    let farm = farm(CellId::new(3, 3), 2);
    for kind in [PickupKind::Punch, PickupKind::Bracelet] {
        let collect = candidate(
            StrategicObjectiveKind::Collect,
            StrategicTarget::Pickup {
                cell: CellId::new(13, 11),
                kind,
            },
            4,
            30,
        );
        assert_eq!(
            choose(baseline, &[baseline, farm, collect], None, 64)
                .0
                .target,
            baseline.target
        );
    }
}

#[test]
fn stalled_alternate_excludes_old_target_before_choosing_first_farm() {
    let baseline = attack(4, 16);
    let old = farm(CellId::new(3, 3), 1);
    let alternate = farm(CellId::new(5, 5), 2);
    let candidates = [baseline, old, alternate];
    assert_eq!(
        choose(baseline, &candidates, Some(old.target), 64).0.target,
        alternate.target
    );
}

#[test]
fn incomplete_scan_spends_its_child_allowance_and_keeps_baseline() {
    let baseline = attack(4, 16);
    let near = farm(CellId::new(3, 3), 2);
    let candidates = [baseline, near, near];
    let (chosen, spent) = choose(baseline, &candidates, None, 16);
    assert_eq!(chosen.target, baseline.target);
    assert_eq!(spent, 1);

    let mut legacy_work = WorkBudget::new(64);
    let legacy = promote_fresh(
        OpeningMode::Legacy,
        baseline,
        &candidates,
        None,
        &mut legacy_work,
    );
    assert_eq!(legacy.target, baseline.target);
    assert_eq!(legacy_work.spent(), 0);
}
