use std::collections::BTreeSet;

use crate::cpu::MobContactLease;
use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::lair_contact::MobContactEnvelope;
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, EntityId, Tick};

use super::motion::{Motion, advance};
use super::pickup_pulse::PickupSafety;
use super::pulse_support::{
    aggregate, aggregate_motion, certify_wait_before_start, contact_envelope, contact_failure,
    guard_teleporter_collection, one_tick, reject_pickups, reject_teleporter_pickups,
    reject_unmodeled_pickups, take_segments, update_origin, validate_origin_forecast,
};
use super::safety::{static_segment_clear, stationary_segment};
use super::types::{PulseLease, PulseRequest};
use super::validation::{self as routing_validation, Context};
use super::{RouteAssurance, RouteFailure, RouteMode};

mod validation;
use validation::{MAX_CONDITIONAL_TICKS, validate_request};

#[derive(Clone, Copy)]
pub(crate) struct PulseOriginGrace {
    pub(crate) cell: CellId,
    pub(crate) bomb_id: EntityId,
}

pub fn certify_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
) -> Result<PulseLease, RouteFailure> {
    if observation.parts().rules.version >= 8 {
        return Err(RouteFailure::InvalidRequest);
    }
    certify_pulse_inner(
        observation,
        forecast,
        request,
        None,
        PickupSafety::Reject,
        None,
        work,
    )
}

pub fn certify_lair_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
) -> Result<(PulseLease, MobContactLease), RouteFailure> {
    let envelope = contact_envelope(observation, request, work)?;
    let lease = certify_pulse_inner(
        observation,
        forecast,
        request,
        None,
        PickupSafety::Reject,
        Some(&envelope),
        work,
    )?;
    Ok((
        lease,
        MobContactLease::new(envelope, request.stopped_safe_until),
    ))
}

pub(crate) fn certify_escape_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: PulseOriginGrace,
    work: &mut WorkBudget,
) -> Result<PulseLease, RouteFailure> {
    validate_origin_forecast(forecast, observation, origin, RouteMode::CertifiedOnly)?;
    certify_pulse_inner(
        observation,
        forecast,
        request,
        Some(origin),
        PickupSafety::Reject,
        None,
        work,
    )
}

pub(crate) fn certify_lair_escape_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: PulseOriginGrace,
    work: &mut WorkBudget,
) -> Result<(PulseLease, MobContactLease), RouteFailure> {
    validate_origin_forecast(forecast, observation, origin, request.mode)?;
    let envelope = contact_envelope(observation, request, work)?;
    let lease = certify_pulse_inner(
        observation,
        forecast,
        request,
        Some(origin),
        PickupSafety::Reject,
        Some(&envelope),
        work,
    )?;
    Ok((
        lease,
        MobContactLease::new(envelope, request.stopped_safe_until),
    ))
}

pub(super) fn certify_pulse_inner(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
    pickup_safety: PickupSafety,
    contact: Option<&MobContactEnvelope>,
    work: &mut WorkBudget,
) -> Result<PulseLease, RouteFailure> {
    let context = routing_validation::validate(
        observation,
        forecast,
        request.start_window,
        request.stopped_safe_until,
    )?;
    validate_request(&context, forecast, request)?;
    let initial = Motion::stopped(&context, request.start_window.start);
    let initial_pits = super::spike_pits::PitTrajectory::from_observation(observation, work)?;
    let mut poses = vec![aggregate(std::slice::from_ref(&initial))?];
    let mut scenarios = Vec::new();
    let mut pit_scenarios = Vec::new();
    let mut assumptions = BTreeSet::new();

    certify_wait_before_start(
        &context,
        forecast,
        request,
        origin,
        pickup_safety != PickupSafety::Reject,
        contact.is_some(),
        work,
        &mut assumptions,
    )?;
    if pickup_safety == PickupSafety::AllowUnmodeledGuard {
        reject_unmodeled_pickups(observation, &stationary_segment(context.actor.position))?;
    }
    if let Some(contact) = contact {
        let last_start = request
            .start_window
            .end
            .0
            .checked_sub(1)
            .map(Tick)
            .ok_or(RouteFailure::InvalidRequest)?;
        if !contact
            .stationary_is_clear_until(context.actor.position, last_start, work)
            .map_err(contact_failure)?
        {
            return Err(RouteFailure::NoRoute);
        }
    }
    for start in request.start_window.start.0..request.start_window.end.0 {
        let start = Tick(start);
        let mut motion = initial.clone();
        let mut pits = initial_pits.clone();
        for tick in super::route_time::add_tick(forecast.basis_tick(), 1)?.0..start.0 {
            super::storm::charge_tick(context.storm, work)?;
            let tick = Tick(tick);
            let segments = advance(&context, &mut motion, MovementIntent::Stop, tick)?;
            pits.advance(tick, segments[0].start(), &segments, work)?;
        }
        scenarios.push((start, motion, origin.is_some()));
        pit_scenarios.push(pits);
    }
    let mut storm_deadlines = scenarios
        .iter()
        .map(|(start, motion, _)| (*start, vec![motion.storm_deadlines()]))
        .collect::<Vec<_>>();
    let mut spike_pit_guards = scenarios
        .iter()
        .zip(&pit_scenarios)
        .map(|((start, _, _), pits)| Ok((*start, vec![pits.guard()?])))
        .collect::<Result<Vec<_>, RouteFailure>>()?;
    for completed in 0..request.hold_ticks {
        let mut next = Vec::with_capacity(scenarios.len());
        let mut next_pits = Vec::with_capacity(pit_scenarios.len());
        for ((start, mut motion, mut origin_active), mut pits) in
            scenarios.into_iter().zip(pit_scenarios)
        {
            let tick = Tick(
                start
                    .0
                    .checked_add(u64::from(completed))
                    .ok_or(RouteFailure::InvalidRequest)?,
            );
            super::storm::charge_tick(context.storm, work)?;
            let segments = advance(&context, &mut motion, request.movement, tick)?;
            pits.advance(tick, segments[0].start(), &segments, work)?;
            take_segments(work, &segments, forecast.dimensions())?;
            if contact.is_some() {
                guard_teleporter_collection(observation, one_tick(tick)?, &segments, work)?;
            }
            match pickup_safety {
                PickupSafety::Reject => reject_pickups(observation, &segments)?,
                PickupSafety::AllowTeleporterGuard => {
                    reject_teleporter_pickups(observation, &segments)?
                }
                PickupSafety::AllowUnmodeledGuard => {
                    reject_unmodeled_pickups(observation, &segments)?
                }
            }
            if let Some(contact) = contact
                && !contact
                    .sweep_is_clear(tick, &segments, work)
                    .map_err(contact_failure)?
            {
                return Err(RouteFailure::NoRoute);
            }
            if !static_segment_clear(observation, &segments) {
                return Err(RouteFailure::NoRoute);
            }
            assumptions.extend(super::safety::check_segments_with_grace(
                forecast,
                &segments,
                one_tick(tick)?,
                request.mode,
                origin.filter(|_| origin_active).map(|value| value.cell),
                None,
            )?);
            update_origin(origin, &segments, &mut origin_active)?;
            next.push((start, motion, origin_active));
            next_pits.push(pits);
        }
        poses.push(aggregate_motion(&next)?);
        for ((_, deadlines), (_, motion, _)) in storm_deadlines.iter_mut().zip(&next) {
            deadlines.push(motion.storm_deadlines());
        }
        for ((_, guards), pits) in spike_pit_guards.iter_mut().zip(&next_pits) {
            guards.push(pits.guard()?);
        }
        scenarios = next;
        pit_scenarios = next_pits;
    }
    certify_stopped_suffix(
        &context,
        forecast,
        request,
        origin,
        &scenarios,
        &pit_scenarios,
        SuffixEvidence {
            work,
            storm_deadlines: &mut storm_deadlines,
            spike_pit_guards: &mut spike_pit_guards,
            assumptions: &mut assumptions,
            guard_teleport: contact.is_some(),
        },
    )?;
    if let Some(contact) = contact {
        let last_safe_update = request
            .stopped_safe_until
            .0
            .checked_sub(1)
            .map(Tick)
            .ok_or(RouteFailure::InvalidRequest)?;
        for (_, motion, _) in &scenarios {
            if !contact
                .stationary_is_clear_until(motion.position, last_safe_update, work)
                .map_err(contact_failure)?
            {
                return Err(RouteFailure::NoRoute);
            }
        }
    }
    if !assumptions.is_empty() && request.hold_ticks > MAX_CONDITIONAL_TICKS {
        return Err(RouteFailure::NoRoute);
    }
    let assurance = if assumptions.is_empty() {
        RouteAssurance::Certified
    } else {
        RouteAssurance::ConditionalStationary {
            bombs: assumptions.into_iter().collect(),
        }
    };
    PulseLease::new(
        context.key,
        forecast.basis_tick(),
        request,
        assurance,
        context.actor.position,
        context.actor.heading,
        poses,
        storm_deadlines,
        spike_pit_guards,
    )
}

struct SuffixEvidence<'a> {
    work: &'a mut WorkBudget,
    storm_deadlines: &'a mut [(Tick, Vec<Option<super::storm::DeadlineBounds>>)],
    spike_pit_guards: &'a mut [(Tick, Vec<super::SpikePitGuard>)],
    assumptions: &'a mut BTreeSet<EntityId>,
    guard_teleport: bool,
}

fn certify_stopped_suffix(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
    scenarios: &[(Tick, Motion, bool)],
    pit_scenarios: &[super::spike_pits::PitTrajectory],
    evidence: SuffixEvidence<'_>,
) -> Result<(), RouteFailure> {
    for ((((start, motion, origin_active), pits), (_, deadlines)), (_, guards)) in scenarios
        .iter()
        .zip(pit_scenarios)
        .zip(evidence.storm_deadlines.iter_mut())
        .zip(evidence.spike_pit_guards.iter_mut())
    {
        let stopped = Tick(
            start
                .0
                .checked_add(u64::from(request.hold_ticks))
                .ok_or(RouteFailure::InvalidRequest)?,
        );
        if stopped >= request.stopped_safe_until {
            continue;
        }
        if evidence.guard_teleport {
            guard_teleporter_collection(
                context.observation,
                TickWindow {
                    start: stopped,
                    end: request.stopped_safe_until,
                },
                &stationary_segment(motion.position),
                evidence.work,
            )?;
        }
        if context.storm.is_none() {
            let mut pits = pits.clone();
            for value in stopped.0..request.stopped_safe_until.0 {
                let tick = Tick(value);
                let segment = stationary_segment(motion.position);
                pits.advance(tick, motion.position, &segment, evidence.work)?;
                guards.push(pits.guard()?);
            }
            let segment = stationary_segment(motion.position);
            take_segments(evidence.work, &segment, forecast.dimensions())?;
            evidence
                .assumptions
                .extend(super::safety::check_segments_with_grace(
                    forecast,
                    &segment,
                    TickWindow {
                        start: stopped,
                        end: request.stopped_safe_until,
                    },
                    request.mode,
                    origin.filter(|_| *origin_active).map(|value| value.cell),
                    None,
                )?);
            continue;
        }
        let mut simulation = motion.clone();
        let mut pits = pits.clone();
        for tick in stopped.0..request.stopped_safe_until.0 {
            super::storm::charge_tick(context.storm, evidence.work)?;
            let tick = Tick(tick);
            let segment = advance(context, &mut simulation, MovementIntent::Stop, tick)?;
            pits.advance(tick, segment[0].start(), &segment, evidence.work)?;
            deadlines.push(simulation.storm_deadlines());
            guards.push(pits.guard()?);
            take_segments(evidence.work, &segment, forecast.dimensions())?;
            evidence
                .assumptions
                .extend(super::safety::check_segments_with_grace(
                    forecast,
                    &segment,
                    one_tick(tick)?,
                    request.mode,
                    origin.filter(|_| *origin_active).map(|value| value.cell),
                    None,
                )?);
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "pulse_origin_tests.rs"]
mod origin_tests;
