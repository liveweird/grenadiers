use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::{Observation, WorkBudget};
use crate::game::Tick;
use crate::game::geometry::{MovementSegment, position_overlaps_cell};

use super::motion::Motion;
use super::pulse::PulseOriginGrace;
use super::pulse_geometry::direction_bit;
use super::safety::footprint_cells;
use super::{PoseBounds, RouteFailure};

pub(super) fn reject_pickups(
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

pub(super) fn aggregate_motion(
    values: &[(Tick, Motion, bool)],
) -> Result<PoseBounds, RouteFailure> {
    let motions = values
        .iter()
        .map(|(_, motion, _)| motion.clone())
        .collect::<Vec<_>>();
    aggregate(&motions)
}

pub(super) fn validate_origin_forecast(
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

pub(super) fn update_origin(
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

pub(super) fn aggregate(values: &[Motion]) -> Result<PoseBounds, RouteFailure> {
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

pub(super) fn take_segments(
    work: &mut WorkBudget,
    segments: &[MovementSegment],
    dimensions: (u16, u16),
) -> Result<(), RouteFailure> {
    let units = u32::try_from(footprint_cells(segments, dimensions).len())
        .map_err(|_| RouteFailure::Capacity)?;
    work.try_take(units.max(1))
        .map_err(|_| RouteFailure::WorkExhausted)
}

pub(super) fn one_tick(tick: Tick) -> Result<TickWindow, RouteFailure> {
    Ok(TickWindow {
        start: tick,
        end: Tick(tick.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?),
    })
}
