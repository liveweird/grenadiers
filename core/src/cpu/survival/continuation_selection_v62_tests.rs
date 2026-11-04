use crate::cpu::WorkBudget;

use super::continuation_scene_fixture::scene;
use super::continuation_selection::OriginRecoveryChoice;
use super::{WORK_ALLOWANCE, plan_continuation_lairs_v62};

#[test]
fn v62_retains_the_complete_stationary_suffix_separately() {
    let scene = scene();
    let choice = plan_continuation_lairs_v62(
        &scene.basis,
        &crate::cpu::forecast::ObservedHazardMemory::new(scene.key),
        scene.basis.parts().tick,
        &crate::cpu::policy::UtilityMemory::new(scene.key),
        &scene.origin,
        &mut WorkBudget::new(WORK_ALLOWANCE),
    )
    .expect("v62 recovery");
    let OriginRecoveryChoice::Continuation(certificate) = choice else {
        panic!("fixture must select retained continuation")
    };
    let (first, last, len) = certificate
        .stationary_suffix_shape_for_test()
        .expect("new-policy suffix");
    assert_eq!(last.0 + 1, certificate.survive_until().0);
    assert_eq!(len as u64, certificate.survive_until().0 - first.0);
    assert!(len <= 840);
}
