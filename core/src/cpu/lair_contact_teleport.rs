use std::collections::BTreeMap;

use crate::cpu::forecast::TickWindow;
use crate::cpu::{BombKnowledge, Observation, WorkBudget};
use crate::game::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, TICKS_PER_SECOND, UNITS_PER_CELL};
use crate::game::{CellId, PickupKind, SpeedLevel, Tick};

use super::{
    ContactEnvelopeError, MobContactEnvelope, distance_to_interval, take_work, travel_units,
};

pub(crate) fn is_clear(
    envelope: &MobContactEnvelope,
    observation: &Observation,
    safe_until: Tick,
    hold_ticks: u8,
    work: &mut WorkBudget,
) -> Result<bool, ContactEnvelopeError> {
    is_clear_sources(
        envelope.sources(),
        observation,
        safe_until,
        hold_ticks,
        work,
    )
}

pub(crate) fn is_clear_sources(
    sources: &[super::Source],
    observation: &Observation,
    safe_until: Tick,
    hold_ticks: u8,
    work: &mut WorkBudget,
) -> Result<bool, ContactEnvelopeError> {
    let parts = observation.parts();
    if !parts.rules.settings.enabled_items().contains_teleporter() {
        return Ok(true);
    }
    if safe_until <= parts.tick {
        return Err(ContactEnvelopeError::InvalidTime);
    }
    take_work(
        work,
        parts
            .pickups
            .len()
            .checked_add(parts.flames.len())
            .ok_or(ContactEnvelopeError::Capacity)?,
    )?;
    let candidates = candidate_cells(observation, safe_until)?;
    if candidates.is_empty() {
        return early_reveal_is_clear(observation, safe_until);
    }
    let pair_count = candidates
        .len()
        .checked_mul(candidates.len().saturating_sub(1))
        .and_then(|pairs| {
            candidates
                .len()
                .checked_mul(sources.len())
                .and_then(|sources| pairs.checked_add(sources))
        })
        .ok_or(ContactEnvelopeError::Capacity)?;
    take_work(work, pair_count)?;
    let residual = residual_travel(hold_ticks)?;
    if candidates.keys().any(|&from| {
        candidates
            .keys()
            .any(|&to| from != to && cells_within(from, to, residual))
    }) {
        return Ok(false);
    }
    let last = Tick(
        safe_until
            .0
            .checked_sub(1)
            .ok_or(ContactEnvelopeError::InvalidTime)?,
    );
    for &cell in candidates.keys() {
        for &source in sources {
            if source.begins_at > last {
                continue;
            }
            let travel = travel_units(source.begins_at, last)?
                .checked_add(residual)
                .ok_or(ContactEnvelopeError::Capacity)?;
            if reaches_landing(source.origin, cell, travel) {
                return Ok(false);
            }
        }
    }
    early_reveal_is_clear(observation, safe_until)
}

pub(crate) fn motion_is_clear(
    observation: &Observation,
    window: TickWindow,
    segments: &[MovementSegment],
    work: &mut WorkBudget,
) -> Result<bool, ContactEnvelopeError> {
    if !observation
        .parts()
        .rules
        .settings
        .enabled_items()
        .contains_teleporter()
    {
        return Ok(true);
    }
    let candidates = candidate_cells(observation, window.end)?;
    take_work(
        work,
        candidates
            .len()
            .checked_mul(segments.len())
            .ok_or(ContactEnvelopeError::Capacity)?,
    )?;
    Ok(!candidates.iter().any(|(&cell, &exposed_at)| {
        exposed_at < window.end && segments.iter().any(|segment| segment.overlaps_cell(cell))
    }))
}

fn candidate_cells(
    observation: &Observation,
    safe_until: Tick,
) -> Result<BTreeMap<CellId, Tick>, ContactEnvelopeError> {
    let parts = observation.parts();
    let mut cells = parts
        .pickups
        .iter()
        .filter_map(|pickup| {
            (pickup.kind == PickupKind::Teleporter).then_some((pickup.cell, parts.tick))
        })
        .collect::<BTreeMap<_, _>>();
    for flame in &parts.flames {
        let expiry = parts
            .tick
            .0
            .checked_add(u64::from(flame.remaining_ticks))
            .ok_or(ContactEnvelopeError::Capacity)?;
        if expiry < safe_until.0 {
            cells
                .entry(flame.cell)
                .and_modify(|current| *current = (*current).min(Tick(expiry)))
                .or_insert(Tick(expiry));
        }
    }
    Ok(cells)
}

fn early_reveal_is_clear(
    observation: &Observation,
    safe_until: Tick,
) -> Result<bool, ContactEnvelopeError> {
    let parts = observation.parts();
    let long_lease = safe_until.0
        > parts
            .tick
            .0
            .checked_add(
                crate::cpu::rules_config(observation)
                    .ok_or(ContactEnvelopeError::Unsupported)?
                    .flame_ticks(),
            )
            .ok_or(ContactEnvelopeError::Capacity)?;
    if !long_lease || parts.arena.blocks.is_empty() {
        return Ok(true);
    }
    Ok(matches!(&parts.bombs, BombKnowledge::Visible(bombs) if bombs.is_empty()))
}

fn residual_travel(hold_ticks: u8) -> Result<i64, ContactEnvelopeError> {
    let numerator = u64::from(SpeedLevel::Four.units_per_second())
        .checked_mul(u64::from(hold_ticks))
        .ok_or(ContactEnvelopeError::Capacity)?;
    i64::try_from(numerator.div_ceil(u64::from(TICKS_PER_SECOND)))
        .map_err(|_| ContactEnvelopeError::Capacity)
}

fn cells_within(from: CellId, to: CellId, distance: i64) -> bool {
    let (from_min_x, from_max_x, from_min_y, from_max_y) = cell_bounds(from);
    let (to_min_x, to_max_x, to_min_y, to_max_y) = cell_bounds(to);
    interval_gap(from_min_x, from_max_x, to_min_x, to_max_x)
        .saturating_add(interval_gap(from_min_y, from_max_y, to_min_y, to_max_y))
        <= distance
}

fn reaches_landing(origin: crate::game::Position, cell: CellId, travel: i64) -> bool {
    let contact = i64::from(HALF_FOOTPRINT) * 2 - 1;
    let (min_x, max_x, min_y, max_y) = cell_bounds(cell);
    distance_to_interval(i64::from(origin.x), min_x - contact, max_x + contact).saturating_add(
        distance_to_interval(i64::from(origin.y), min_y - contact, max_y + contact),
    ) <= travel
}

fn cell_bounds(cell: CellId) -> (i64, i64, i64, i64) {
    let min_x = i64::from(cell.column) * i64::from(UNITS_PER_CELL);
    let min_y = i64::from(cell.row) * i64::from(UNITS_PER_CELL);
    (
        min_x,
        min_x + i64::from(UNITS_PER_CELL) - 1,
        min_y,
        min_y + i64::from(UNITS_PER_CELL) - 1,
    )
}

fn interval_gap(a_min: i64, a_max: i64, b_min: i64, b_max: i64) -> i64 {
    if a_max < b_min {
        b_min - a_max
    } else if b_max < a_min {
        a_min - b_max
    } else {
        0
    }
}
