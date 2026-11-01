use crate::cpu::forecast::HazardForecast;
use crate::cpu::routing::{ContinuationRequest, NominalLeg, RouteFailure};
use crate::cpu::{MemoryKey, Observation, WorkBudget};
use crate::game::Tick;

use super::continuation_test_fixture::{displaced_forecast, rebased_scene, unsupported_scenes};
use super::{PlantEscapeOrigin, WORK_ALLOWANCE, certify_continuation};

pub(super) fn assert_guard_negatives(
    basis: &Observation,
    hazards: &HazardForecast,
    request: ContinuationRequest,
    legs: &[NominalLeg],
    origin: &PlantEscapeOrigin,
    key: MemoryKey,
) {
    assert!(matches!(
        certify_continuation(
            basis,
            hazards,
            ContinuationRequest {
                survive_until: Tick(210),
                ..request
            },
            legs,
            origin,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ),
        Err(RouteFailure::InvalidRequest)
    ));
    for unsupported in unsupported_scenes(basis) {
        assert!(matches!(
            certify_continuation(
                &unsupported,
                hazards,
                request,
                legs,
                origin,
                &mut WorkBudget::new(WORK_ALLOWANCE),
            ),
            Err(RouteFailure::Unsupported) | Err(RouteFailure::IdentityMismatch)
        ));
    }
    assert!(matches!(
        certify_continuation(
            &rebased_scene(basis),
            hazards,
            request,
            legs,
            origin,
            &mut WorkBudget::new(WORK_ALLOWANCE),
        ),
        Err(RouteFailure::IdentityMismatch)
    ));
    let (displaced, mut work) = displaced_forecast(basis, key);
    assert_eq!(displaced.stationary_assumptions().len(), 1);
    assert!(matches!(
        certify_continuation(basis, &displaced, request, legs, origin, &mut work),
        Err(RouteFailure::Unsupported)
    ));
}
