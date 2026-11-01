use crate::cpu::forecast::HazardForecast;
use crate::cpu::routing::{
    ContinuationCertificate, ContinuationRequest, NominalLeg, PulseOriginGrace,
};
use crate::cpu::{Observation, WorkBudget};

use super::PlantEscapeOrigin;

/// Revalidates the opaque plant origin before handing its private collision
/// proof to the inactive continuation guidance kernel.
pub(crate) fn certify_continuation(
    observation: &Observation,
    forecast: &HazardForecast,
    request: ContinuationRequest,
    nominal_legs: &[NominalLeg],
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Result<ContinuationCertificate, crate::cpu::routing::RouteFailure> {
    charge_origin_validation(origin, observation, work)?;
    origin
        .retain_stopped(observation)
        .ok_or(crate::cpu::routing::RouteFailure::IdentityMismatch)?;
    crate::cpu::routing::certify_continuation(
        observation,
        forecast,
        request,
        nominal_legs,
        PulseOriginGrace {
            cell: origin.cell(),
            bomb_id: origin.bomb_id(),
        },
        work,
    )
}

#[cfg(test)]
pub(super) fn certify_nominal_without_stop_branches(
    observation: &Observation,
    forecast: &HazardForecast,
    request: ContinuationRequest,
    nominal_legs: &[NominalLeg],
    origin: &PlantEscapeOrigin,
    work: &mut WorkBudget,
) -> Result<ContinuationCertificate, crate::cpu::routing::RouteFailure> {
    charge_origin_validation(origin, observation, work)?;
    origin
        .retain_stopped(observation)
        .ok_or(crate::cpu::routing::RouteFailure::IdentityMismatch)?;
    crate::cpu::routing::certify_nominal_for_test(
        observation,
        forecast,
        request,
        nominal_legs,
        PulseOriginGrace {
            cell: origin.cell(),
            bomb_id: origin.bomb_id(),
        },
        work,
    )
}

pub(super) fn charge_origin_validation(
    origin: &PlantEscapeOrigin,
    current: &Observation,
    work: &mut WorkBudget,
) -> Result<(), crate::cpu::routing::RouteFailure> {
    let old = origin.basis.parts();
    let new = current.parts();
    let metadata = old
        .actors
        .len()
        .checked_add(new.actors.len())
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(crate::cpu::routing::RouteFailure::Capacity)?;
    work.try_take(metadata)
        .map_err(|_| crate::cpu::routing::RouteFailure::WorkExhausted)?;
    let old_effects = old
        .actors
        .iter()
        .find(|actor| actor.seat() == old.recipient)
        .and_then(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor) => Some(actor.effects.len()),
            crate::cpu::ObservedActor::Hidden { .. } => None,
        })
        .ok_or(crate::cpu::routing::RouteFailure::IdentityMismatch)?;
    let new_effects = new
        .actors
        .iter()
        .find(|actor| actor.seat() == new.recipient)
        .and_then(|actor| match actor {
            crate::cpu::ObservedActor::Visible(actor) => Some(actor.effects.len()),
            crate::cpu::ObservedActor::Hidden { .. } => None,
        })
        .ok_or(crate::cpu::routing::RouteFailure::IdentityMismatch)?;
    let crate::cpu::BombKnowledge::Visible(new_bombs) = &new.bombs else {
        return Err(crate::cpu::routing::RouteFailure::IdentityMismatch);
    };
    let effect_width = old_effects
        .checked_add(new_effects)
        .ok_or(crate::cpu::routing::RouteFailure::Capacity)?;
    let nested_effects = new_effects
        .checked_mul(effect_width)
        .ok_or(crate::cpu::routing::RouteFailure::Capacity)?;
    let units = 32_usize
        .checked_add(
            old.actors
                .len()
                .checked_mul(2)
                .ok_or(crate::cpu::routing::RouteFailure::Capacity)?,
        )
        .and_then(|value| value.checked_add(new.actors.len().checked_mul(4)?))
        .and_then(|value| value.checked_add(old.arena.terrain.len().max(new.arena.terrain.len())))
        .and_then(|value| value.checked_add(new.arena.blocks.len()))
        .and_then(|value| value.checked_add(new.oil.len()))
        .and_then(|value| value.checked_add(new_bombs.len().checked_mul(2)?))
        .and_then(|value| value.checked_add(old_effects))
        .and_then(|value| value.checked_add(new_effects))
        .and_then(|value| value.checked_add(nested_effects))
        .and_then(|value| {
            value.checked_add(usize::try_from(origin.certificate.contact_actor_work()?).ok()?)
        })
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(crate::cpu::routing::RouteFailure::Capacity)?;
    work.try_take(units)
        .map_err(|_| crate::cpu::routing::RouteFailure::WorkExhausted)
}
