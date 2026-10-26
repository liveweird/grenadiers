use std::collections::BTreeSet;

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::geometry::MovementSegment;
use crate::game::{EntityId, Tick};

use super::motion::{Motion, advance};
use super::pulse_geometry::direction_bit;
use super::safety::{check_segments, footprint_cells, static_segment_clear, stationary_segment};
use super::types::{PoseBounds, PulseLease, PulseRequest};
use super::validation::{self, Context};
use super::{RouteAssurance, RouteFailure};

const MAX_PULSE_TICKS: u8 = 12;
const MAX_CONDITIONAL_TICKS: u8 = 3;
const MAX_ACCEPTED_AGE: u64 = 6;
const STOPPED_PUBLICATION_TICKS: u64 = 3;

pub fn certify_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
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

    certify_wait_before_start(&context, forecast, request, work, &mut assumptions)?;
    for start in request.start_window.start.0..request.start_window.end.0 {
        scenarios.push((Tick(start), initial.clone()));
    }
    for completed in 0..request.hold_ticks {
        let mut next = Vec::with_capacity(scenarios.len());
        for (start, mut motion) in scenarios {
            let tick = Tick(
                start
                    .0
                    .checked_add(u64::from(completed))
                    .ok_or(RouteFailure::InvalidRequest)?,
            );
            let segments = advance(&context, &mut motion, request.movement, tick)?;
            take_segments(work, &segments)?;
            reject_pickups(observation, &segments)?;
            if !static_segment_clear(observation, &segments) {
                return Err(RouteFailure::NoRoute);
            }
            assumptions.extend(check_segments(
                forecast,
                &segments,
                one_tick(tick)?,
                request.mode,
            )?);
            next.push((start, motion));
        }
        poses.push(aggregate_motion(&next)?);
        scenarios = next;
    }
    certify_stopped_suffix(forecast, request, work, &scenarios, &mut assumptions)?;
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

fn certify_wait_before_start(
    context: &Context<'_>,
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
    assumptions: &mut BTreeSet<EntityId>,
) -> Result<(), RouteFailure> {
    let end = request.start_window.end;
    let start = Tick(forecast.basis_tick().0 + 1);
    if start >= end {
        return Ok(());
    }
    let segment = stationary_segment(context.actor.position);
    take_segments(work, &segment)?;
    reject_pickups(context.observation, &segment)?;
    assumptions.extend(check_segments(
        forecast,
        &segment,
        TickWindow { start, end },
        request.mode,
    )?);
    Ok(())
}

fn certify_stopped_suffix(
    forecast: &HazardForecast,
    request: PulseRequest,
    work: &mut WorkBudget,
    scenarios: &[(Tick, Motion)],
    assumptions: &mut BTreeSet<EntityId>,
) -> Result<(), RouteFailure> {
    for (start, motion) in scenarios {
        let stopped = Tick(
            start
                .0
                .checked_add(u64::from(request.hold_ticks))
                .ok_or(RouteFailure::InvalidRequest)?,
        );
        if stopped >= request.stopped_safe_until {
            continue;
        }
        let segment = stationary_segment(motion.position);
        take_segments(work, &segment)?;
        assumptions.extend(check_segments(
            forecast,
            &segment,
            TickWindow {
                start: stopped,
                end: request.stopped_safe_until,
            },
            request.mode,
        )?);
    }
    Ok(())
}

fn reject_pickups(
    observation: &Observation,
    segments: &[MovementSegment],
) -> Result<(), RouteFailure> {
    if observation.parts().pickups.iter().any(|pickup| {
        segments
            .iter()
            .any(|segment| segment.overlaps_cell(pickup.cell))
    }) {
        Err(RouteFailure::Unsupported)
    } else {
        Ok(())
    }
}

fn aggregate_motion(values: &[(Tick, Motion)]) -> Result<PoseBounds, RouteFailure> {
    let motions = values
        .iter()
        .map(|(_, motion)| motion.clone())
        .collect::<Vec<_>>();
    aggregate(&motions)
}

fn aggregate(values: &[Motion]) -> Result<PoseBounds, RouteFailure> {
    let first = values.first().ok_or(RouteFailure::Capacity)?;
    let mut min = first.position;
    let mut max = first.position;
    let mut mask = 0;
    for value in values {
        min.x = min.x.min(value.position.x);
        min.y = min.y.min(value.position.y);
        max.x = max.x.max(value.position.x);
        max.y = max.y.max(value.position.y);
        mask |= direction_bit(value.heading);
    }
    PoseBounds::new(min, max, mask).ok_or(RouteFailure::Capacity)
}

fn take_segments(work: &mut WorkBudget, segments: &[MovementSegment]) -> Result<(), RouteFailure> {
    let units =
        u32::try_from(footprint_cells(segments).len()).map_err(|_| RouteFailure::Capacity)?;
    work.try_take(units.max(1))
        .map_err(|_| RouteFailure::WorkExhausted)
}

fn one_tick(tick: Tick) -> Result<TickWindow, RouteFailure> {
    Ok(TickWindow {
        start: tick,
        end: Tick(tick.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?),
    })
}
