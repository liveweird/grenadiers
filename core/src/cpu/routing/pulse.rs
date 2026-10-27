use std::collections::BTreeSet;

use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{MovementIntent, Observation, WorkBudget};
use crate::game::geometry::MovementSegment;
use crate::game::geometry::position_overlaps_cell;
use crate::game::{CellId, EntityId, Tick};

use super::motion::{Motion, advance};
use super::pulse_geometry::direction_bit;
use super::safety::{footprint_cells, static_segment_clear, stationary_segment};
use super::types::{PoseBounds, PulseLease, PulseRequest};
use super::validation::{self, Context};
use super::{RouteAssurance, RouteFailure};

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
    certify_pulse_inner(observation, forecast, request, None, work)
}

pub(crate) fn certify_escape_pulse(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: PulseOriginGrace,
    work: &mut WorkBudget,
) -> Result<PulseLease, RouteFailure> {
    validate_origin_forecast(forecast, origin)?;
    certify_pulse_inner(observation, forecast, request, Some(origin), work)
}

fn certify_pulse_inner(
    observation: &Observation,
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
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

    certify_wait_before_start(&context, forecast, request, origin, work, &mut assumptions)?;
    for start in request.start_window.start.0..request.start_window.end.0 {
        scenarios.push((Tick(start), initial.clone(), origin.is_some()));
    }
    for completed in 0..request.hold_ticks {
        let mut next = Vec::with_capacity(scenarios.len());
        for (start, mut motion, mut origin_active) in scenarios {
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
        scenarios = next;
    }
    certify_stopped_suffix(
        forecast,
        request,
        origin,
        work,
        &scenarios,
        &mut assumptions,
    )?;
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
    origin: Option<PulseOriginGrace>,
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
    assumptions.extend(super::safety::check_segments_with_grace(
        forecast,
        &segment,
        TickWindow { start, end },
        request.mode,
        origin.map(|value| value.cell),
        None,
    )?);
    Ok(())
}

fn certify_stopped_suffix(
    forecast: &HazardForecast,
    request: PulseRequest,
    origin: Option<PulseOriginGrace>,
    work: &mut WorkBudget,
    scenarios: &[(Tick, Motion, bool)],
    assumptions: &mut BTreeSet<EntityId>,
) -> Result<(), RouteFailure> {
    for (start, motion, origin_active) in scenarios {
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
        assumptions.extend(super::safety::check_segments_with_grace(
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

fn aggregate_motion(values: &[(Tick, Motion, bool)]) -> Result<PoseBounds, RouteFailure> {
    let motions = values
        .iter()
        .map(|(_, motion, _)| motion.clone())
        .collect::<Vec<_>>();
    aggregate(&motions)
}

fn validate_origin_forecast(
    forecast: &HazardForecast,
    origin: PulseOriginGrace,
) -> Result<(), RouteFailure> {
    let cell = forecast
        .cell(origin.cell)
        .ok_or(RouteFailure::InvalidRequest)?;
    if cell.uncertainty.iter().any(|uncertain| {
        matches!(uncertain.kind, crate::cpu::forecast::HazardKind::Blocking)
            && !matches!(
                uncertain.reason,
                crate::cpu::forecast::UncertaintyReason::BombMotion(id) if id == origin.bomb_id
            )
    }) {
        return Err(RouteFailure::NoRoute);
    }
    Ok(())
}

fn update_origin(
    origin: Option<PulseOriginGrace>,
    segments: &[MovementSegment],
    active: &mut bool,
) -> Result<(), RouteFailure> {
    let Some(origin) = origin else {
        return Ok(());
    };
    for segment in segments {
        let start_inside = position_overlaps_cell(segment.start(), origin.cell);
        if *active && !start_inside {
            *active = false;
        }
        if !*active && segment.overlaps_cell(origin.cell) {
            return Err(RouteFailure::NoRoute);
        }
        if *active && !position_overlaps_cell(segment.end(), origin.cell) {
            *active = false;
        }
    }
    Ok(())
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

#[cfg(test)]
mod origin_tests {
    use super::*;
    use crate::game::Position;

    #[test]
    fn origin_grace_is_consumed_on_exit_and_cannot_be_reentered() {
        let origin = PulseOriginGrace {
            cell: CellId::new(2, 1),
            bomb_id: EntityId(7),
        };
        let inside = Position::center(origin.cell);
        let outside = Position::center(CellId::new(3, 1));
        let mut active = true;
        update_origin(
            Some(origin),
            &[MovementSegment::new(inside, outside)],
            &mut active,
        )
        .unwrap();
        assert!(!active);
        assert_eq!(
            update_origin(
                Some(origin),
                &[MovementSegment::new(outside, inside)],
                &mut active,
            ),
            Err(RouteFailure::NoRoute)
        );
    }
}
