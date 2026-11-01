use std::collections::BTreeSet;

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::lair_contact::MobContactEnvelope;
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{EntityId, MovementSegment, Tick};

use super::continuation_proof::{
    ProofContext, ProofState, advance_checked, certify_stationary_suffix, certify_stop_branch,
    reject_initial_scene,
};
use super::continuation_types::{
    ContinuationCertificate, ContinuationParts, ContinuationRequest, MAX_CONTINUATION_LEGS,
    MAX_CONTINUATION_MOVES, NominalLeg,
};
use super::motion::Motion;
use super::pulse::PulseOriginGrace;
use super::pulse_support::{contact_failure, validate_origin_forecast};
use super::validation;
use super::{RouteFailure, RouteMode};

/// Certifies complete conditional guidance only. It grants no movement and
/// deliberately does not provide the fresh short-window Teleporter/contact
/// obligation required by a future execution grant.
pub(crate) fn certify_continuation(
    observation: &Observation,
    forecast: &HazardForecast,
    request: ContinuationRequest,
    nominal_legs: &[NominalLeg],
    origin: PulseOriginGrace,
    work: &mut WorkBudget,
) -> Result<ContinuationCertificate, RouteFailure> {
    certify_continuation_inner(
        observation,
        forecast,
        request,
        nominal_legs,
        origin,
        true,
        work,
    )
}

#[cfg(test)]
pub(crate) fn certify_nominal_for_test(
    observation: &Observation,
    forecast: &HazardForecast,
    request: ContinuationRequest,
    nominal_legs: &[NominalLeg],
    origin: PulseOriginGrace,
    work: &mut WorkBudget,
) -> Result<ContinuationCertificate, RouteFailure> {
    certify_continuation_inner(
        observation,
        forecast,
        request,
        nominal_legs,
        origin,
        false,
        work,
    )
}

#[allow(clippy::too_many_arguments)]
fn certify_continuation_inner(
    observation: &Observation,
    forecast: &HazardForecast,
    request: ContinuationRequest,
    nominal_legs: &[NominalLeg],
    origin: PulseOriginGrace,
    certify_stop_branches: bool,
    work: &mut WorkBudget,
) -> Result<ContinuationCertificate, RouteFailure> {
    reject_initial_scene(observation, forecast, work)?;
    validate_legs(nominal_legs)?;
    let nominal_updates = nominal_legs.iter().try_fold(0_u64, |total, leg| {
        total
            .checked_add(u64::from(leg.updates()))
            .ok_or(RouteFailure::Capacity)
    })?;
    let nominal_end = request
        .first_update
        .0
        .checked_add(nominal_updates)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)?;
    let start_end = Tick(
        request
            .first_update
            .0
            .checked_add(1)
            .ok_or(RouteFailure::InvalidRequest)?,
    );
    let context = validation::validate(
        observation,
        forecast,
        TickWindow {
            start: request.first_update,
            end: start_end,
        },
        request.survive_until,
    )?;
    if request.settled_stop_tick > observation.parts().tick
        || request.survive_until <= request.first_update
        || nominal_end > request.survive_until
    {
        return Err(RouteFailure::InvalidRequest);
    }
    validate_origin_forecast(
        forecast,
        observation,
        origin,
        RouteMode::AllowStationaryAssumptions,
    )?;
    let contact = MobContactEnvelope::new(observation, work).map_err(contact_failure)?;
    let proof = ProofContext {
        context: &context,
        forecast,
        contact: &contact,
        origin,
    };
    let mut state = ProofState {
        motion: Motion::stopped(&context, observation.parts().tick),
        pits: super::spike_pits::PitTrajectory::from_observation(observation, work)?,
        origin_active: true,
    };
    let mut assumptions = BTreeSet::<EntityId>::new();
    certify_stationary_suffix(
        &proof,
        &mut state,
        Tick(
            observation
                .parts()
                .tick
                .0
                .checked_add(1)
                .ok_or(RouteFailure::InvalidRequest)?,
        ),
        request.first_update,
        work,
        &mut assumptions,
    )?;
    let admission_tick = Tick(
        request
            .first_update
            .0
            .checked_sub(1)
            .ok_or(RouteFailure::InvalidRequest)?,
    );
    let admission_checkpoint = super::continuation_types::ContinuationCheckpoint {
        tick: admission_tick,
        movement: MovementIntent::Stop,
        motion: state.motion.snapshot(),
        segments: vec![MovementSegment::new(
            state.motion.position,
            state.motion.position,
        )],
        origin_active: state.origin_active,
        pit_guard: state.pits.guard()?,
    };
    let mut checkpoints = Vec::new();
    let mut stop_branches = Vec::new();
    let mut tick = request.first_update;
    for leg in nominal_legs {
        for _ in 0..leg.updates() {
            let movement = leg.movement();
            let checkpoint =
                advance_checked(&proof, &mut state, movement, tick, work, &mut assumptions)?;
            if certify_stop_branches {
                stop_branches.push(certify_stop_branch(
                    &proof,
                    &state,
                    tick,
                    work,
                    &mut assumptions,
                )?);
            }
            checkpoints.push(checkpoint);
            tick = Tick(tick.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?);
        }
    }
    certify_stationary_suffix(
        &proof,
        &mut state,
        tick,
        request.survive_until,
        work,
        &mut assumptions,
    )?;
    ContinuationCertificate::from_parts(ContinuationParts {
        key: context.key,
        basis: observation.clone(),
        basis_tick: forecast.basis_tick(),
        request,
        origin,
        admission_checkpoint,
        legs: nominal_legs.to_vec(),
        checkpoints,
        stop_branches,
        assumptions,
        evolution: forecast.evolution().clone(),
        contact,
    })
    .ok_or(RouteFailure::Capacity)
}

pub(super) fn validate_legs(legs: &[NominalLeg]) -> Result<(), RouteFailure> {
    if legs.is_empty() || legs.len() > MAX_CONTINUATION_LEGS {
        return Err(RouteFailure::Capacity);
    }
    let mut moving = 0_u16;
    for leg in legs {
        if leg.updates() == 0 {
            return Err(RouteFailure::InvalidRequest);
        }
        if let NominalLeg::Move { updates, .. } = leg {
            moving = moving
                .checked_add(u16::from(*updates))
                .ok_or(RouteFailure::Capacity)?;
        }
    }
    if moving == 0 || moving > MAX_CONTINUATION_MOVES {
        return Err(RouteFailure::Capacity);
    }
    Ok(())
}
