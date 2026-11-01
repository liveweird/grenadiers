use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::cpu::lair_contact::{ContactEnvelopeError, distance_to_interval};
use crate::cpu::{Observation, ObservedActor, VisibleActor, WorkBudget};
use crate::game::geometry::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, UNITS_PER_CELL};
use crate::game::{CellId, Position, SpeedLevel, Tick};

use super::RouteFailure;
use super::safety::footprint_cells;
use super::teleporter_step::{take, usize_to_work};
use super::validation::Context;

#[derive(Clone, Copy)]
pub(super) struct Region {
    pub(super) min: Position,
    pub(super) max: Position,
}

pub(super) fn reject_pits(
    context: &Context<'_>,
    segments: &[MovementSegment],
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    if let Some(pits) = context.observation.spike_pits() {
        let cells = charged_footprint_cells(
            segments,
            (
                context.observation.parts().arena.width,
                context.observation.parts().arena.height,
            ),
            work,
        )?;
        let scans = cells
            .len()
            .checked_mul(pits.pits().len())
            .ok_or(RouteFailure::Capacity)?;
        take(work, usize_to_work(scans)?)?;
        if cells
            .iter()
            .any(|cell| pits.pits().iter().any(|pit| pit.cell == *cell))
        {
            return Err(RouteFailure::NoRoute);
        }
    }
    Ok(())
}

pub(super) fn reject_storm_region(
    context: &Context<'_>,
    region: Region,
    begins: Tick,
    ends: Tick,
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    take(work, 1)?;
    let Some(environment) = context.observation.environment() else {
        return Ok(());
    };
    let Some(cloud) = environment.storm_cloud() else {
        return Ok(());
    };
    let elapsed_end = ends.0 - context.observation.parts().tick.0;
    if elapsed_end > u64::from(cloud.leg_remaining_ticks) {
        return Err(RouteFailure::NoRoute);
    }
    let elapsed_start = begins.0 - context.observation.parts().tick.0;
    let start = translate_cloud(cloud.position, cloud.direction, elapsed_start);
    let end = translate_cloud(cloud.position, cloud.direction, elapsed_end);
    let extent = crate::game::STORM_CLOUD_HALF_EXTENT + HALF_FOOTPRINT;
    let cloud_region = Region {
        min: Position {
            x: start.x.min(end.x) - extent,
            y: start.y.min(end.y) - extent,
        },
        max: Position {
            x: start.x.max(end.x) + extent,
            y: start.y.max(end.y) + extent,
        },
    };
    if boxes_overlap(region, cloud_region) {
        return Err(RouteFailure::NoRoute);
    }
    Ok(())
}

pub(super) fn region_around(position: Position, distance: i64) -> Result<Region, RouteFailure> {
    let distance = i32::try_from(distance).map_err(|_| RouteFailure::Capacity)?;
    Ok(Region {
        min: Position {
            x: position.x.saturating_sub(distance),
            y: position.y.saturating_sub(distance),
        },
        max: Position {
            x: position.x.saturating_add(distance),
            y: position.y.saturating_add(distance),
        },
    })
}

pub(super) fn charged_region_cells(
    region: Region,
    dimensions: (u16, u16),
    work: &mut WorkBudget,
) -> Result<Vec<CellId>, RouteFailure> {
    charged_footprint_cells(
        &[MovementSegment::new(region.min, region.max)],
        dimensions,
        work,
    )
}

pub(super) fn partial_endpoints(
    initial: Position,
    segments: &[MovementSegment],
    work: &mut WorkBudget,
) -> Result<Vec<Position>, RouteFailure> {
    take(work, 1)?;
    let mut points = vec![initial];
    for segment in segments {
        let dx = segment.end().x - segment.start().x;
        let dy = segment.end().y - segment.start().y;
        let distance = dx.abs().max(dy.abs());
        take(
            work,
            u32::try_from(distance).map_err(|_| RouteFailure::Capacity)? + 2,
        )?;
        for offset in 0..=distance {
            points.push(Position {
                x: segment.start().x + dx.signum() * offset,
                y: segment.start().y + dy.signum() * offset,
            });
        }
    }
    Ok(points)
}

pub(super) fn charge_segment_scans(
    forecast: &HazardForecast,
    segments: &[MovementSegment],
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let cells = charged_footprint_cells(segments, forecast.dimensions(), work)?;
    for cell in cells {
        take(work, 1)?;
        let cell = forecast.cell(cell).ok_or(RouteFailure::InvalidRequest)?;
        let scans = cell
            .lethal
            .len()
            .checked_add(cell.blocking.len())
            .and_then(|value| value.checked_add(cell.uncertainty.len()))
            .ok_or(RouteFailure::Capacity)?;
        take(work, usize_to_work(scans)?)?;
    }
    Ok(())
}

pub(super) fn charge_static_scans(
    observation: &Observation,
    segments: &[MovementSegment],
    work: &mut WorkBudget,
) -> Result<(), RouteFailure> {
    let parts = observation.parts();
    let cells = charged_footprint_cells(segments, (parts.arena.width, parts.arena.height), work)?;
    let per_cell = parts
        .arena
        .blocks
        .len()
        .checked_add(parts.oil.len())
        .and_then(|value| value.checked_add(1))
        .ok_or(RouteFailure::Capacity)?;
    let scans = cells
        .len()
        .checked_mul(per_cell)
        .ok_or(RouteFailure::Capacity)?;
    take(work, usize_to_work(scans)?)
}

fn charged_footprint_cells(
    segments: &[MovementSegment],
    dimensions: (u16, u16),
    work: &mut WorkBudget,
) -> Result<Vec<CellId>, RouteFailure> {
    take(work, usize_to_work(segments.len())?)?;
    let (width, height) = dimensions;
    let mut candidates = 0_usize;
    for segment in segments {
        let min_x = i64::from(segment.start().x.min(segment.end().x) - HALF_FOOTPRINT);
        let min_y = i64::from(segment.start().y.min(segment.end().y) - HALF_FOOTPRINT);
        let max_x = i64::from(segment.start().x.max(segment.end().x) + HALF_FOOTPRINT - 1);
        let max_y = i64::from(segment.start().y.max(segment.end().y) + HALF_FOOTPRINT - 1);
        let last_column = i64::from(width) - 1;
        let last_row = i64::from(height) - 1;
        let first_column = min_x
            .div_euclid(i64::from(UNITS_PER_CELL))
            .clamp(0, last_column);
        let final_column = max_x
            .div_euclid(i64::from(UNITS_PER_CELL))
            .clamp(0, last_column);
        let first_row = min_y
            .div_euclid(i64::from(UNITS_PER_CELL))
            .clamp(0, last_row);
        let final_row = max_y
            .div_euclid(i64::from(UNITS_PER_CELL))
            .clamp(0, last_row);
        let columns =
            usize::try_from(final_column - first_column + 1).map_err(|_| RouteFailure::Capacity)?;
        let rows =
            usize::try_from(final_row - first_row + 1).map_err(|_| RouteFailure::Capacity)?;
        candidates = candidates
            .checked_add(columns.checked_mul(rows).ok_or(RouteFailure::Capacity)?)
            .ok_or(RouteFailure::Capacity)?;
    }
    take(work, usize_to_work(candidates)?)?;
    Ok(footprint_cells(segments, dimensions))
}

pub(super) fn distance_to_cell(position: Position, cell: CellId) -> i64 {
    let left = i64::from(cell.column) * i64::from(UNITS_PER_CELL) - i64::from(HALF_FOOTPRINT);
    let top = i64::from(cell.row) * i64::from(UNITS_PER_CELL) - i64::from(HALF_FOOTPRINT);
    let width = i64::from(UNITS_PER_CELL + 2 * HALF_FOOTPRINT);
    distance_to_interval(i64::from(position.x), left, left + width)
        + distance_to_interval(i64::from(position.y), top, top + width)
}

pub(super) fn max_travel(ticks: u64) -> Result<i64, RouteFailure> {
    let units = u64::from(SpeedLevel::Four.units_per_second())
        .checked_mul(ticks)
        .ok_or(RouteFailure::Capacity)?
        .div_ceil(u64::from(TICKS_PER_SECOND))
        .checked_add(1)
        .ok_or(RouteFailure::Capacity)?;
    i64::try_from(units).map_err(|_| RouteFailure::Capacity)
}

fn translate_cloud(position: Position, direction: crate::game::Direction, ticks: u64) -> Position {
    let distance = i32::try_from(ticks)
        .unwrap_or(i32::MAX)
        .saturating_mul(crate::game::STORM_CLOUD_SPEED_PER_TICK);
    super::pulse_geometry::translated(position, direction, distance)
}

fn boxes_overlap(a: Region, b: Region) -> bool {
    a.min.x <= b.max.x && b.min.x <= a.max.x && a.min.y <= b.max.y && b.min.y <= a.max.y
}

pub(super) fn visible_alive(actor: &ObservedActor) -> Option<&VisibleActor> {
    match actor {
        ObservedActor::Visible(actor) if actor.alive => Some(actor),
        _ => None,
    }
}

pub(super) fn intersects(left: TickWindow, right: TickWindow) -> bool {
    left.start.0 < right.end.0 && right.start.0 < left.end.0
}

pub(super) fn one_tick(tick: Tick) -> Result<TickWindow, RouteFailure> {
    Ok(TickWindow {
        start: tick,
        end: next_tick(tick)?,
    })
}

pub(super) fn next_tick(tick: Tick) -> Result<Tick, RouteFailure> {
    tick.0
        .checked_add(1)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)
}

pub(super) fn previous_tick(tick: Tick) -> Result<Tick, RouteFailure> {
    tick.0
        .checked_sub(1)
        .map(Tick)
        .ok_or(RouteFailure::InvalidRequest)
}

pub(super) fn contact_failure(error: ContactEnvelopeError) -> RouteFailure {
    match error {
        ContactEnvelopeError::WorkExhausted => RouteFailure::WorkExhausted,
        ContactEnvelopeError::Capacity => RouteFailure::Capacity,
        ContactEnvelopeError::Unsupported | ContactEnvelopeError::InvalidTime => {
            RouteFailure::Unsupported
        }
    }
}
