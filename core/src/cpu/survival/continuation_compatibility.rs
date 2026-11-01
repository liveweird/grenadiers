//! Charged public-fact compatibility for inactive continuation renewal.

use crate::cpu::forecast::ForecastEvolution;
use crate::cpu::routing::{
    CONTINUATION_RENEWAL_ALLOWANCE, ContinuationCertificate, PreparedFrame, RouteFailure,
};
use crate::cpu::{Observation, WorkBudget};

pub(super) fn actor_compatible_charged(
    basis: &Observation,
    current: &Observation,
    evolution: &ForecastEvolution,
    work: &mut WorkBudget,
) -> bool {
    let old = basis.parts();
    let new = current.parts();
    let metadata = old
        .actors
        .len()
        .checked_add(new.actors.len())
        .and_then(|value| u32::try_from(value).ok());
    if metadata.is_none_or(|units| work.try_take(units).is_err()) {
        return false;
    }
    let (Some(before), Some(after)) = (
        super::compatibility::actor(basis),
        super::compatibility::actor(current),
    ) else {
        return false;
    };
    let Some(effect_width) = before.effects.len().checked_add(after.effects.len()) else {
        return false;
    };
    let effects = after
        .effects
        .len()
        .checked_mul(effect_width)
        .and_then(|value| value.checked_add(before.effects.len()))
        .and_then(|value| value.checked_add(after.effects.len()));
    let scans = old
        .actors
        .len()
        .checked_add(new.actors.len())
        .and_then(|value| value.checked_add(effects?))
        .and_then(|value| u32::try_from(value.checked_add(8)?).ok());
    if scans.is_none_or(|units| work.try_take(units).is_err()) {
        return false;
    }
    let Some(expected) =
        evolution.continuation_expected_live_bombs(before.id, current.parts().tick, work)
    else {
        return false;
    };
    super::compatibility::actor_compatible(basis, current, Some(expected), before.weapon)
}

pub(super) fn rivals_allow_segments_charged(
    observation: &Observation,
    segments: impl IntoIterator<Item = crate::game::MovementSegment>,
    work: &mut WorkBudget,
) -> bool {
    for segment in segments {
        let Some(units) = u32::try_from(observation.parts().actors.len()).ok() else {
            return false;
        };
        if work.try_take(units).is_err()
            || !super::compatibility::visible_rivals_allow_segment(
                observation,
                segment.start(),
                segment.end(),
            )
        {
            return false;
        }
    }
    true
}

pub(super) fn prepare_checked(
    certificate: &ContinuationCertificate,
    current: &Observation,
    work: &mut WorkBudget,
) -> Result<PreparedFrame, RouteFailure> {
    let mut bounded = WorkBudget::new(work.remaining().min(CONTINUATION_RENEWAL_ALLOWANCE));
    let result = (|| {
        if !actor_compatible_charged(
            certificate.basis(),
            current,
            certificate.evolution(),
            &mut bounded,
        ) {
            return Err(RouteFailure::NoRoute);
        }
        let frame = certificate.prepare_frame(current, &mut bounded)?;
        if let PreparedFrame::Grant(frame) = &frame
            && !rivals_allow_segments_charged(current, frame.segments(), &mut bounded)
        {
            return Err(RouteFailure::NoRoute);
        }
        Ok(frame)
    })();
    work.try_take(bounded.spent())
        .map_err(|_| RouteFailure::WorkExhausted)?;
    result
}
