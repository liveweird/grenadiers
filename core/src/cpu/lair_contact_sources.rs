use crate::game::{CellId, Position, Tick};
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
