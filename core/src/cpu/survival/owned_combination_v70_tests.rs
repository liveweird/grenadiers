use crate::cpu::WorkBudget;

use super::owned_combination_fixture::basis;
use super::{
    WORK_ALLOWANCE, plan_hard_combination_v69_with_handoff, plan_hard_combination_v70_with_handoff,
};

#[test]
fn hard70_combination_identity_preserves_the_v69_proof_and_work() {
    let value = basis();
    let mut old_work = WorkBudget::new(WORK_ALLOWANCE);
    let mut new_work = WorkBudget::new(WORK_ALLOWANCE);
    let old = plan_hard_combination_v69_with_handoff(
        &value.observation,
        &value.memory,
        &value.history,
        value.settled,
        &value.handoff,
        &mut old_work,
    )
    .unwrap()
    .expect("v69 curated combination");
    let new = plan_hard_combination_v70_with_handoff(
        &value.observation,
        &value.memory,
        &value.history,
        value.settled,
        &value.handoff,
        &mut new_work,
    )
    .unwrap()
    .expect("v70 curated combination");
    assert_eq!(new.survival, old.survival);
    assert_eq!(format!("{:?}", new.contact), format!("{:?}", old.contact));
    assert_eq!(new_work.spent(), old_work.spent());
}
