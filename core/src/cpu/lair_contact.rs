use crate::game::geometry::MovementSegment;
use crate::game::types::{HALF_FOOTPRINT, TICKS_PER_SECOND};
use crate::game::{EntityId, MOB_UNITS_PER_SECOND, Position, Tick};
use crate::room::LairConfig;

use super::{Observation, WorkBudget};

const SPAWN_DIRECTIONS: [(i16, i16); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContactEnvelopeError {
    Unsupported,
    InvalidTime,
    WorkExhausted,
    Capacity,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SourceKind {
    Existing(EntityId),
    PossibleSpawn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Source {
    kind: SourceKind,
    origin: Position,
    begins_at: Tick,
}

/// Finite wall-agnostic envelopes for at most 16 mobs and eight spawn origins.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MobContactEnvelope {
    world: super::WorldId,
    basis_tick: Tick,
    config: LairConfig,
    sources: Vec<Source>,
}

impl MobContactEnvelope {
    pub(crate) fn new(
        observation: &Observation,
        work: &mut WorkBudget,
    ) -> Result<Self, ContactEnvelopeError> {
        let observed = observation
            .lairs()
            .ok_or(ContactEnvelopeError::Unsupported)?;
        let source_count = observed
            .mobs()
            .len()
            .checked_add(observed.lairs().len().saturating_mul(4))
            .ok_or(ContactEnvelopeError::Capacity)?;
        take_work(work, source_count)?;
        let mut sources = Vec::with_capacity(source_count);
        sources.extend(observed.mobs().iter().map(|mob| Source {
            kind: SourceKind::Existing(mob.id),
            origin: mob.position,
            begins_at: observation.parts().tick,
        }));
        for lair in observed.lairs() {
            for (dc, dr) in SPAWN_DIRECTIONS {
                let column = i32::from(lair.cell.column)
                    .checked_add(i32::from(dc))
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or(ContactEnvelopeError::Capacity)?;
                let row = i32::from(lair.cell.row)
                    .checked_add(i32::from(dr))
                    .and_then(|value| u16::try_from(value).ok())
                    .ok_or(ContactEnvelopeError::Capacity)?;
                sources.push(Source {
                    kind: SourceKind::PossibleSpawn,
                    origin: Position::center(crate::game::CellId::new(column, row)),
                    begins_at: lair.next_spawn_at,
                });
            }
        }
        Ok(Self {
            world: observation.parts().world,
            basis_tick: observation.parts().tick,
            config: observed.config(),
            sources,
        })
    }

    /// Checks actual piecewise actor motion at one update after mob movement.
    pub(crate) fn sweep_is_clear(
        &self,
        tick: Tick,
        segments: &[MovementSegment],
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        if tick <= self.basis_tick {
            return Err(ContactEnvelopeError::InvalidTime);
        }
        take_work(
            work,
            self.sources
                .len()
                .checked_mul(segments.len())
                .ok_or(ContactEnvelopeError::Capacity)?,
        )?;
        for source in &self.sources {
            if tick < source.begins_at {
                continue;
            }
            let travel = travel_units(source.begins_at, tick)?;
            if segments
                .iter()
                .any(|segment| source_reaches_segment(*source, travel, *segment))
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Stationary reachability is monotone, so the interval coalesces to its end.
    pub(crate) fn stationary_is_clear_until(
        &self,
        position: Position,
        end: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        self.sweep_is_clear(end, &[MovementSegment::new(position, position)], work)
    }

    /// Allows ordinary motion/death and predicted births while rejecting facts
    /// outside the original finite envelope or beyond the lease deadline.
    #[allow(dead_code, reason = "consumed by the following CPU19 lease milestone")]
    pub(crate) fn contains(
        &self,
        current: &Observation,
        valid_until: Tick,
        work: &mut WorkBudget,
    ) -> Result<bool, ContactEnvelopeError> {
        if current.parts().world != self.world
            || current.parts().tick < self.basis_tick
            || current.parts().tick >= valid_until
        {
            return Ok(false);
        }
        let Some(lairs) = current.lairs() else {
            return Ok(false);
        };
        if lairs.config() != self.config {
            return Ok(false);
        }
        take_work(
            work,
            lairs
                .mobs()
                .len()
                .checked_mul(self.sources.len())
                .ok_or(ContactEnvelopeError::Capacity)?,
        )?;
        for mob in lairs.mobs() {
            let existing = self
                .sources
                .iter()
                .find(|source| matches!(source.kind, SourceKind::Existing(id) if id == mob.id));
            let fits = existing.map_or_else(
                || {
                    self.sources.iter().any(|source| {
                        source.kind == SourceKind::PossibleSpawn
                            && position_fits(*source, current.parts().tick, mob.position)
                                .unwrap_or(false)
                    })
                },
                |source| {
                    position_fits(*source, current.parts().tick, mob.position).unwrap_or(false)
                },
            );
            if !fits {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

/// Lets a queued Easy driver suppress an optional pause near NPC/spawn danger.
#[must_use]
pub fn has_urgent_lair_threat(
    observation: &Observation,
    through: Tick,
    work: &mut WorkBudget,
) -> bool {
    let Some(actor) = observation
        .parts()
        .actors
        .iter()
        .find_map(|actor| match actor {
            super::ObservedActor::Visible(actor) if actor.seat == observation.parts().recipient => {
                Some(actor)
            }
            _ => None,
        })
    else {
        return true;
    };
    let Ok(envelope) = MobContactEnvelope::new(observation, work) else {
        return true;
    };
    !matches!(
        envelope.stationary_is_clear_until(actor.position, through, work),
        Ok(true)
    )
}

fn take_work(work: &mut WorkBudget, units: usize) -> Result<(), ContactEnvelopeError> {
    work.try_take(u32::try_from(units).map_err(|_| ContactEnvelopeError::Capacity)?)
        .map_err(|_| ContactEnvelopeError::WorkExhausted)
}

fn travel_units(from: Tick, through: Tick) -> Result<i64, ContactEnvelopeError> {
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

fn source_reaches_segment(source: Source, travel: i64, segment: MovementSegment) -> bool {
    let contact_offset = i64::from(HALF_FOOTPRINT) * 2 - 1;
    let min_x = i64::from(segment.start().x.min(segment.end().x)) - contact_offset;
    let max_x = i64::from(segment.start().x.max(segment.end().x)) + contact_offset;
    let min_y = i64::from(segment.start().y.min(segment.end().y)) - contact_offset;
    let max_y = i64::from(segment.start().y.max(segment.end().y)) + contact_offset;
    distance_to_interval(i64::from(source.origin.x), min_x, max_x).saturating_add(
        distance_to_interval(i64::from(source.origin.y), min_y, max_y),
    ) <= travel
}

#[allow(dead_code, reason = "consumed by the following CPU19 lease milestone")]
fn position_fits(
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

fn distance_to_interval(value: i64, min: i64, max: i64) -> i64 {
    if value < min {
        min - value
    } else if value > max {
        value - max
    } else {
        0
    }
}

#[cfg(test)]
#[path = "lair_contact_tests.rs"]
mod tests;
