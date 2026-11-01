use crate::game::geometry::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, TICKS_PER_SECOND};
use crate::game::{CellId, MOB_UNITS_PER_SECOND, Position, Tick};
use crate::room::RespawnLairConfig;

use super::{
    ContactEnvelopeError, LairIdentity, Observation, Source, SourceKind, WorkBudget, take_work,
};

const DIRECTIONS: [(i16, i16); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

pub(super) fn build(
    observation: &Observation,
    work: &mut WorkBudget,
) -> Result<(LairIdentity, Vec<Source>), ContactEnvelopeError> {
    let (config, mobs, lairs) = if let Some(observed) = observation.lairs() {
        (
            LairIdentity::Legacy(observed.config()),
            observed.mobs(),
            observed
                .lairs()
                .iter()
                .map(|lair| (lair.cell, lair.next_spawn_at))
                .collect::<Vec<_>>(),
        )
    } else if let Some(observed) = observation.respawn_lairs() {
        let RespawnLairConfig::Enabled(settings) = observed.config() else {
            return Err(ContactEnvelopeError::Unsupported);
        };
        let replacement = observation
            .parts()
            .tick
            .0
            .checked_add(1)
            .and_then(|tick| tick.checked_add(settings.spawn_ticks()))
            .map(Tick)
            .ok_or(ContactEnvelopeError::Capacity)?;
        (
            LairIdentity::Respawn(observed.config()),
            observed.mobs(),
            observed
                .lairs()
                .iter()
                .map(|lair| (lair.cell, lair.next_spawn_at.unwrap_or(replacement)))
                .collect::<Vec<_>>(),
        )
    } else {
        return Err(ContactEnvelopeError::Unsupported);
    };
    let count = mobs
        .len()
        .checked_add(lairs.len().saturating_mul(DIRECTIONS.len()))
        .ok_or(ContactEnvelopeError::Capacity)?;
    take_work(work, count)?;
    let mut sources = Vec::with_capacity(count);
    sources.extend(mobs.iter().map(|mob| Source {
        kind: SourceKind::Existing {
            id: mob.id,
            lair: mob.lair,
        },
        origin: mob.position,
        begins_at: observation.parts().tick,
    }));
    for (index, (cell, begins_at)) in lairs.into_iter().enumerate() {
        add_spawn_sources(index, cell, begins_at, &mut sources)?;
    }
    Ok((config, sources))
}

fn add_spawn_sources(
    index: usize,
    cell: CellId,
    begins_at: Tick,
    sources: &mut Vec<Source>,
) -> Result<(), ContactEnvelopeError> {
    let lair = u8::try_from(index).map_err(|_| ContactEnvelopeError::Capacity)?;
    for (dc, dr) in DIRECTIONS {
        let column = i32::from(cell.column)
            .checked_add(i32::from(dc))
            .and_then(|value| u16::try_from(value).ok())
            .ok_or(ContactEnvelopeError::Capacity)?;
        let row = i32::from(cell.row)
            .checked_add(i32::from(dr))
            .and_then(|value| u16::try_from(value).ok())
            .ok_or(ContactEnvelopeError::Capacity)?;
        sources.push(Source {
            kind: SourceKind::PossibleSpawn { lair },
            origin: Position::center(CellId::new(column, row)),
            begins_at,
        });
    }
    Ok(())
}

pub(crate) fn travel_units(from: Tick, through: Tick) -> Result<i64, ContactEnvelopeError> {
    let ticks = through
        .0
        .checked_sub(from.0)
        .ok_or(ContactEnvelopeError::InvalidTime)?;
    let numerator = u128::from(MOB_UNITS_PER_SECOND)
        .checked_mul(u128::from(ticks))
        .ok_or(ContactEnvelopeError::Capacity)?;
    let travel = numerator.div_ceil(u128::from(TICKS_PER_SECOND));
    i64::try_from(travel).map_err(|_| ContactEnvelopeError::Capacity)
}

pub(crate) fn source_reaches_segment(
    source: Source,
    travel: i64,
    segment: MovementSegment,
) -> bool {
    let contact_offset = i64::from(HALF_FOOTPRINT) * 2 - 1;
    let min_x = i64::from(segment.start().x.min(segment.end().x)) - contact_offset;
    let max_x = i64::from(segment.start().x.max(segment.end().x)) + contact_offset;
    let min_y = i64::from(segment.start().y.min(segment.end().y)) - contact_offset;
    let max_y = i64::from(segment.start().y.max(segment.end().y)) + contact_offset;
    distance_to_interval(i64::from(source.origin.x), min_x, max_x).saturating_add(
        distance_to_interval(i64::from(source.origin.y), min_y, max_y),
    ) <= travel
}

pub(super) fn position_fits(
    source: Source,
    tick: Tick,
    position: Position,
) -> Result<bool, ContactEnvelopeError> {
    if tick < source.begins_at {
        return Ok(false);
    }
    let travel = u64::try_from(travel_units(source.begins_at, tick)?)
        .map_err(|_| ContactEnvelopeError::Capacity)?;
    let distance = i64::from(source.origin.x)
        .abs_diff(i64::from(position.x))
        .checked_add(i64::from(source.origin.y).abs_diff(i64::from(position.y)))
        .ok_or(ContactEnvelopeError::Capacity)?;
    Ok(distance <= travel)
}

pub(crate) fn distance_to_interval(value: i64, min: i64, max: i64) -> i64 {
    if value < min {
        min - value
    } else if value > max {
        value - max
    } else {
        0
    }
}
