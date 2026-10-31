use std::collections::BTreeSet;

use crate::cpu::MobContactLease;
use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::lair_contact::MobContactEnvelope;
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::{CellId, EntityId, Tick};

use super::motion::{Motion, advance};
use super::pulse_support::{
    aggregate, aggregate_motion, certify_wait_before_start, contact_envelope, contact_failure,
    guard_teleporter_collection, one_tick, reject_pickups, reject_teleporter_pickups,
    take_segments, update_origin, validate_origin_forecast,
};
use super::safety::{static_segment_clear, stationary_segment};
use super::types::{PulseLease, PulseRequest};
use super::validation::{self, Context};
use super::{RouteAssurance, RouteFailure, RouteMode};

const MAX_PULSE_TICKS: u8 = 12;
const MAX_CONDITIONAL_TICKS: u8 = 3;
const MAX_ACCEPTED_AGE: u64 = 6;
const STOPPED_PUBLICATION_TICKS: u64 = 3;

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
    certify_pulse_inner(observation, forecast, request, None, false, None, work)
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
        false,
        Some(&envelope),
        work,
    )?;
    Ok((
        lease,
        MobContactLease::new(envelope, request.stopped_safe_until),
    ))
}

pub(crate) fn certify_lair_contact_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
) -> Result<MobContactLease, RouteFailure> {
    let envelope = contact_envelope(observation, request, work)?;
    let _ = certify_pulse_inner(
        observation,
        forecast,
        request,
        None,
        true,
        Some(&envelope),
        work,
    )?;
    Ok(MobContactLease::new(envelope, request.stopped_safe_until))
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
        false,
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
        false,
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
    allow_pickups: bool,
    contact: Option<&MobContactEnvelope>,
    work: &mut WorkBudget,
) -> Result<PulseLease, RouteFailure> {
    let context = validation::validate(
        observation,
        forecast,
        request.start_window,
        request.stopped_safe_until,
    )?;
    validate_request(&context, forecast, request)?;
    let initial = Motion::stopped(&context, request.start_window.start);
    let mut poses = vec![aggregate(std::slice::from_ref(&initial))?];
    let mut scenarios = Vec::new();
    let mut assumptions = BTreeSet::new();

    certify_wait_before_start(
        &context,
        forecast,
        request,
        origin,
        allow_pickups,
        contact.is_some(),
        work,
        &mut assumptions,
    )?;
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
        for tick in super::route_time::add_tick(forecast.basis_tick(), 1)?.0..start.0 {
            super::storm::charge_tick(context.storm, work)?;
            let _ = advance(&context, &mut motion, MovementIntent::Stop, Tick(tick))?;
        }
        scenarios.push((start, motion, origin.is_some()));
    }
    let mut storm_deadlines = scenarios
        .iter()
        .map(|(start, motion, _)| (*start, vec![motion.storm_deadlines()]))
        .collect::<Vec<_>>();
    for completed in 0..request.hold_ticks {
        let mut next = Vec::with_capacity(scenarios.len());
        for (start, mut motion, mut origin_active) in scenarios {
            let tick = Tick(
                start
                    .0
                    .checked_add(u64::from(completed))
                    .ok_or(RouteFailure::InvalidRequest)?,
            );
            super::storm::charge_tick(context.storm, work)?;
            let segments = advance(&context, &mut motion, request.movement, tick)?;
            take_segments(work, &segments, forecast.dimensions())?;
            if contact.is_some() {
                guard_teleporter_collection(observation, one_tick(tick)?, &segments, work)?;
            }
            if !allow_pickups {
                reject_pickups(observation, &segments)?;
            } else {
                reject_teleporter_pickups(observation, &segments)?;
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
        }
        poses.push(aggregate_motion(&next)?);
        for ((_, deadlines), (_, motion, _)) in storm_deadlines.iter_mut().zip(&next) {
            deadlines.push(motion.storm_deadlines());
        }
        scenarios = next;
    }
    certify_stopped_suffix(
        &context,
        forecast,
        request,
        origin,
        &scenarios,
        SuffixEvidence {
            work,
            storm_deadlines: &mut storm_deadlines,
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
    )
}

fn validate_request(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: PulseRequest,
) -> Result<(), RouteFailure> {
    let basis = forecast.basis_tick().0;
    let latest_start_end = basis
        .checked_add(MAX_ACCEPTED_AGE + 2)
        .ok_or(RouteFailure::InvalidRequest)?;
    let latest_stop = request
        .start_window
        .end
        .0
        .checked_sub(1)
        .and_then(|tick| tick.checked_add(u64::from(request.hold_ticks)))
        .ok_or(RouteFailure::InvalidRequest)?;
    let required_lease = latest_stop
        .checked_add(STOPPED_PUBLICATION_TICKS + MAX_ACCEPTED_AGE + 1)
        .ok_or(RouteFailure::InvalidRequest)?;
    if request.hold_ticks == 0
        || request.hold_ticks > MAX_PULSE_TICKS
        || request.start_window.start.0 <= basis
        || request.start_window.end.0 > latest_start_end
        || request.settled_stop_tick > forecast.basis_tick()
        || request.stopped_safe_until.0 < required_lease
        || request.stopped_safe_until > forecast.horizon_end()
        || (request.movement == MovementIntent::Stop && request.hold_ticks != 1)
    {
        return Err(RouteFailure::InvalidRequest);
    }
    if request.settled_stop_tick.0 > context.observation.parts().tick.0 {
        return Err(RouteFailure::InvalidRequest);
    }
    Ok(())
}

struct SuffixEvidence<'a> {
    work: &'a mut WorkBudget,
    storm_deadlines: &'a mut [(Tick, Vec<Option<super::storm::DeadlineBounds>>)],
    assumptions: &'a mut BTreeSet<EntityId>,
    guard_teleport: bool,
}

fn certify_stopped_suffix(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
    scenarios: &[(Tick, Motion, bool)],
    evidence: SuffixEvidence<'_>,
) -> Result<(), RouteFailure> {
    for ((start, motion, origin_active), (_, deadlines)) in
        scenarios.iter().zip(evidence.storm_deadlines.iter_mut())
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
        for tick in stopped.0..request.stopped_safe_until.0 {
            super::storm::charge_tick(context.storm, evidence.work)?;
            let tick = Tick(tick);
            let segment = advance(context, &mut simulation, MovementIntent::Stop, tick)?;
            deadlines.push(simulation.storm_deadlines());
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
