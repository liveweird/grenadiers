use std::collections::BTreeSet;

use crate::cpu::Observation;
use crate::cpu::forecast::{HazardForecast, TickWindow, UncertaintyReason};
use crate::game::arena::{ARENA_HEIGHT, ARENA_WIDTH};
use crate::game::geometry::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, UNITS_PER_CELL};
use crate::game::{CellId, EntityId, OilBarrelState, Position, Terrain};

use super::{RouteFailure, RouteMode};

pub(super) fn static_segment_clear(
    observation: &Observation,
    segments: &[MovementSegment],
) -> bool {
    footprint_cells(segments)
        .into_iter()
        .all(|cell| !static_blocked(observation, cell))
}

pub(super) fn check_segments(
    forecast: &HazardForecast,
    segments: &[MovementSegment],
    time: TickWindow,
    mode: RouteMode,
) -> Result<BTreeSet<EntityId>, RouteFailure> {
    check_segments_with_grace(forecast, segments, time, mode, None)
}

pub(super) fn check_segments_with_grace(
    forecast: &HazardForecast,
    segments: &[MovementSegment],
    time: TickWindow,
    mode: RouteMode,
    blocking_grace: Option<CellId>,
) -> Result<BTreeSet<EntityId>, RouteFailure> {
    let mut assumptions = BTreeSet::new();
    for cell_id in footprint_cells(segments) {
        let cell = forecast.cell(cell_id).ok_or(RouteFailure::InvalidRequest)?;
        if cell.lethal.iter().any(|window| intersects(*window, time))
            || (blocking_grace != Some(cell_id)
                && cell.blocking.iter().any(|window| intersects(*window, time)))
        {
            return Err(RouteFailure::NoRoute);
        }
        for uncertain in &cell.uncertainty {
            if !intersects(uncertain.window, time) {
                continue;
            }
            match (mode, uncertain.reason) {
                (RouteMode::AllowStationaryAssumptions, UncertaintyReason::BombMotion(id))
                    if forecast.stationary_assumptions().contains(&id) =>
                {
                    assumptions.insert(id);
                    if assumptions.len() > crate::cpu::MAX_OBSERVED_BOMBS {
                        return Err(RouteFailure::Capacity);
                    }
                }
                _ => return Err(RouteFailure::NoRoute),
            }
        }
    }
    Ok(assumptions)
}

pub(super) fn footprint_cells(segments: &[MovementSegment]) -> Vec<CellId> {
    let mut cells = BTreeSet::new();
    for segment in segments {
        let min_x = segment.start().x.min(segment.end().x) - HALF_FOOTPRINT;
        let min_y = segment.start().y.min(segment.end().y) - HALF_FOOTPRINT;
        let max_x = segment.start().x.max(segment.end().x) + HALF_FOOTPRINT - 1;
        let max_y = segment.start().y.max(segment.end().y) + HALF_FOOTPRINT - 1;
        let first_column = min_x
            .div_euclid(UNITS_PER_CELL)
            .clamp(0, i32::from(ARENA_WIDTH) - 1);
        let last_column = max_x
            .div_euclid(UNITS_PER_CELL)
            .clamp(0, i32::from(ARENA_WIDTH) - 1);
        let first_row = min_y
            .div_euclid(UNITS_PER_CELL)
            .clamp(0, i32::from(ARENA_HEIGHT) - 1);
        let last_row = max_y
            .div_euclid(UNITS_PER_CELL)
            .clamp(0, i32::from(ARENA_HEIGHT) - 1);
        for row in first_row..=last_row {
            for column in first_column..=last_column {
                let cell = CellId::new(column as u16, row as u16);
                if segment.overlaps_cell(cell) {
                    cells.insert(cell);
                }
            }
        }
    }
    cells.into_iter().collect()
}

pub(super) fn stationary_segment(position: Position) -> [MovementSegment; 1] {
    [MovementSegment::new(position, position)]
}

pub(super) fn static_blocked(observation: &Observation, cell: CellId) -> bool {
    let parts = observation.parts();
    let index = usize::from(cell.row) * usize::from(ARENA_WIDTH) + usize::from(cell.column);
    parts.arena.terrain[index] == Terrain::Wall
        || parts.arena.blocks.contains(&cell)
        || parts
            .oil
            .iter()
            .any(|oil| oil.cell == cell && oil.state == OilBarrelState::Unlit)
}

fn intersects(left: TickWindow, right: TickWindow) -> bool {
    left.start.0 < right.end.0 && right.start.0 < left.end.0
}
