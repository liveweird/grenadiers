use std::collections::BTreeSet;

use crate::cpu::WorkBudget;
use crate::cpu::forecast::{HazardForecast, TickWindow};
use crate::game::{CellId, EntityId, MovementSegment, Position, Tick};

use super::safety::{
    RecipientBarrier, check_segments_with_grace, footprint_cells, static_segment_clear,
    stationary_segment,
};
use super::validation::Context;
use super::{RouteFailure, RouteMode, RouteRequest};

#[derive(Clone, Copy)]
pub(super) struct CollisionProofs {
    pub(super) placement_origin: Option<CellId>,
    pub(super) recipient_barrier: Option<RecipientBarrier>,
}

pub(super) fn stationary_risk(
    forecast: &HazardForecast,
    lanes: &[(Position, Tick)],
    end: Tick,
    mode: RouteMode,
    work: &mut WorkBudget,
) -> Result<u32, RouteFailure> {
    let mut risk = 0_u32;
    for (position, start) in lanes {
        let window = TickWindow { start: *start, end };
        for cell_id in footprint_cells(&stationary_segment(*position)) {
            work.try_take(1).map_err(|_| RouteFailure::WorkExhausted)?;
            let cell = forecast.cell(cell_id).ok_or(RouteFailure::InvalidRequest)?;
            let exact = cell
                .lethal
                .iter()
                .chain(&cell.blocking)
                .filter(|hazard| intersects(**hazard, window))
                .count() as u32;
            let uncertain = cell
                .uncertainty
                .iter()
                .filter(|hazard| {
                    intersects(hazard.window, window)
                        && !matches!(
                            (mode, hazard.reason),
                            (
                                RouteMode::AllowStationaryAssumptions,
                                crate::cpu::forecast::UncertaintyReason::BombMotion(id)
                            ) if forecast.stationary_assumptions().contains(&id)
                        )
                })
                .count() as u32;
            let hazards = exact.saturating_add(uncertain);
            if hazards > 0 {
                let center = Position::center(cell_id);
                let distance = position.x.abs_diff(center.x) + position.y.abs_diff(center.y);
                let proximity = 4_096_u32.saturating_sub(distance);
                risk = risk
                    .saturating_add(hazards.saturating_mul(10_000))
                    .saturating_add(proximity);
            }
        }
    }
    Ok(risk)
}

pub(super) fn check_transition_all(
    context: &Context<'_>,
    forecasts: &[&HazardForecast],
    segments: &[MovementSegment],
    window: TickWindow,
    mode: RouteMode,
    proofs: CollisionProofs,
    work: &mut WorkBudget,
) -> Result<BTreeSet<EntityId>, RouteFailure> {
    let units = footprint_cells(segments)
        .len()
        .checked_mul(forecasts.len())
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(units.max(1))
        .map_err(|_| RouteFailure::WorkExhausted)?;
    if !static_segment_clear(context.observation, segments)
        || context.observation.parts().pickups.iter().any(|pickup| {
            segments
                .iter()
                .any(|segment| segment.overlaps_cell(pickup.cell))
        })
    {
        return Err(RouteFailure::NoRoute);
    }
    let mut assumptions = BTreeSet::new();
    for forecast in forecasts {
        assumptions.extend(check_segments_with_grace(
            forecast,
            segments,
            window,
            mode,
            proofs.placement_origin,
            proofs.recipient_barrier,
        )?);
    }
    Ok(assumptions)
}

pub(super) fn check_stationary_all(
    forecasts: &[&HazardForecast],
    position: Position,
    window: TickWindow,
    mode: RouteMode,
    proofs: CollisionProofs,
    work: &mut WorkBudget,
) -> Result<BTreeSet<EntityId>, RouteFailure> {
    if window.start >= window.end {
        return Ok(BTreeSet::new());
    }
    let segment = stationary_segment(position);
    let units = footprint_cells(&segment)
        .len()
        .checked_mul(forecasts.len())
        .and_then(|count| u32::try_from(count).ok())
        .ok_or(RouteFailure::Capacity)?;
    work.try_take(units)
        .map_err(|_| RouteFailure::WorkExhausted)?;
    let mut assumptions = BTreeSet::new();
    for forecast in forecasts {
        assumptions.extend(check_segments_with_grace(
            forecast,
            &segment,
            window,
            mode,
            proofs.placement_origin,
            proofs.recipient_barrier,
        )?);
    }
    Ok(assumptions)
}

pub(super) fn wait_boundaries(
    context: &Context<'_>,
    forecast: &HazardForecast,
    work: &mut WorkBudget,
) -> Result<Vec<Tick>, RouteFailure> {
    let intervals = forecast
        .cells()
        .iter()
        .map(|cell| cell.lethal.len() + cell.blocking.len() + cell.uncertainty.len())
        .sum::<usize>();
    let units = forecast.cells().len().saturating_add(intervals);
    work.try_take(u32::try_from(units).map_err(|_| RouteFailure::Capacity)?)
        .map_err(|_| RouteFailure::WorkExhausted)?;
    let mut boundaries = context
        .slow_expiries
        .iter()
        .copied()
        .chain(context.reverse_expiry)
        .chain(forecast.cells().iter().flat_map(|cell| {
            cell.lethal
                .iter()
                .chain(&cell.blocking)
                .map(|window| window.end)
                .chain(cell.uncertainty.iter().map(|window| window.window.end))
        }))
        .collect::<Vec<_>>();
    boundaries.sort_unstable();
    boundaries.dedup();
    Ok(boundaries)
}

pub(super) fn wait_lengths(
    boundaries: &[Tick],
    request: RouteRequest,
    ready_ticks: &[Tick],
) -> Result<Vec<u16>, RouteFailure> {
    let earliest = ready_ticks
        .iter()
        .copied()
        .min()
        .ok_or(RouteFailure::Capacity)?;
    let latest = ready_ticks
        .iter()
        .copied()
        .max()
        .ok_or(RouteFailure::Capacity)?;
    let mut waits = boundaries
        .iter()
        .copied()
        .filter_map(|end| end.0.checked_sub(earliest.0))
        .filter(|delay| *delay > 0)
        .filter_map(|delay| u16::try_from(delay).ok())
        .collect::<Vec<_>>();
    if let Some(delay) = request.survive_until.0.checked_sub(latest.0)
        && let Ok(delay) = u16::try_from(delay)
        && delay > 0
    {
        waits.push(delay);
    }
    waits.sort_unstable();
    waits.dedup();
    waits.truncate(2);
    Ok(waits)
}

pub(super) fn one_tick(tick: Tick) -> Result<TickWindow, RouteFailure> {
    Ok(TickWindow {
        start: tick,
        end: Tick(tick.0.checked_add(1).ok_or(RouteFailure::InvalidRequest)?),
    })
}

fn intersects(left: TickWindow, right: TickWindow) -> bool {
    left.start < right.end && right.start < left.end
}
