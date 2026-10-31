use std::collections::BTreeSet;

use crate::room::SpikePitSettings;

use super::{Arena, CellId, MovementSegment, MovementTrace, Position, Terrain, Tick};

pub const SPIKE_PIT_LAYOUT_VERSION: u16 = 1;
pub const MAX_SPIKE_PITS: usize = 4;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SpikePitPhase {
    #[default]
    Idle,
    Warning,
    Raised,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpikePitClockError {
    DeadlineOverflow,
    NonMonotonicUpdate { at: Tick, last_updated_at: Tick },
    PastTickQuery { at: Tick, last_updated_at: Tick },
}

/// The half-open interval during which one armed pit is raised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpikePitRaisedInterval {
    pub starts_at: Tick,
    pub ends_at: Tick,
}

impl SpikePitRaisedInterval {
    #[must_use]
    pub const fn contains(self, at: Tick) -> bool {
        self.starts_at.0 <= at.0 && at.0 < self.ends_at.0
    }

    #[must_use]
    pub const fn overlaps(self, starts_at: Tick, ends_at: Tick) -> bool {
        starts_at.0 < ends_at.0 && starts_at.0 < self.ends_at.0 && self.starts_at.0 < ends_at.0
    }
}

/// One pit's checked half-open timing state.
///
/// Contact at `T` warns on `[T, T + W)`, raises on
/// `[T + W, T + W + R)`, and is idle at the retraction deadline.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum SpikePitClockState {
    #[default]
    Idle,
    Warning {
        rises_at: Tick,
        retracts_at: Tick,
    },
    Raised {
        rises_at: Tick,
        retracts_at: Tick,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SpikePitClock {
    state: SpikePitClockState,
    last_retraction_update: Option<Tick>,
    last_updated_at: Option<Tick>,
}

impl SpikePitClock {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: SpikePitClockState::Idle,
            last_retraction_update: None,
            last_updated_at: None,
        }
    }

    #[must_use]
    pub const fn phase(self) -> SpikePitPhase {
        match self.state {
            SpikePitClockState::Idle => SpikePitPhase::Idle,
            SpikePitClockState::Warning { .. } => SpikePitPhase::Warning,
            SpikePitClockState::Raised { .. } => SpikePitPhase::Raised,
        }
    }

    /// Forecasts the phase without mutating the authoritative clock.
    pub const fn phase_at(self, at: Tick) -> Result<SpikePitPhase, SpikePitClockError> {
        if let Some(last_updated_at) = self.last_updated_at
            && at.0 < last_updated_at.0
        {
            return Err(SpikePitClockError::PastTickQuery {
                at,
                last_updated_at,
            });
        }
        Ok(match self.state {
            SpikePitClockState::Idle => SpikePitPhase::Idle,
            SpikePitClockState::Warning {
                rises_at,
                retracts_at,
            } => {
                if at.0 >= retracts_at.0 {
                    SpikePitPhase::Idle
                } else if at.0 >= rises_at.0 {
                    SpikePitPhase::Raised
                } else {
                    SpikePitPhase::Warning
                }
            }
            SpikePitClockState::Raised { retracts_at, .. } => {
                if at.0 >= retracts_at.0 {
                    SpikePitPhase::Idle
                } else {
                    SpikePitPhase::Raised
                }
            }
        })
    }

    #[must_use]
    pub const fn next_deadline(self) -> Option<Tick> {
        match self.state {
            SpikePitClockState::Idle => None,
            SpikePitClockState::Warning { rises_at, .. } => Some(rises_at),
            SpikePitClockState::Raised { retracts_at, .. } => Some(retracts_at),
        }
    }

    /// Remaining ticks in the phase forecast at `at`.
    pub const fn remaining_ticks(self, at: Tick) -> Result<u64, SpikePitClockError> {
        if let Some(last_updated_at) = self.last_updated_at
            && at.0 < last_updated_at.0
        {
            return Err(SpikePitClockError::PastTickQuery {
                at,
                last_updated_at,
            });
        }
        Ok(match self.state {
            SpikePitClockState::Idle => 0,
            SpikePitClockState::Warning {
                rises_at,
                retracts_at,
            } => {
                if at.0 < rises_at.0 {
                    rises_at.0 - at.0
                } else {
                    retracts_at.0.saturating_sub(at.0)
                }
            }
            SpikePitClockState::Raised { retracts_at, .. } => retracts_at.0.saturating_sub(at.0),
        })
    }

    #[must_use]
    pub const fn raised_interval(self) -> Option<SpikePitRaisedInterval> {
        match self.state {
            SpikePitClockState::Idle => None,
            SpikePitClockState::Warning {
                rises_at,
                retracts_at,
            }
            | SpikePitClockState::Raised {
                rises_at,
                retracts_at,
            } => Some(SpikePitRaisedInterval {
                starts_at: rises_at,
                ends_at: retracts_at,
            }),
        }
    }

    /// Resolves deadlines and then applies contact without extending an active cycle.
    ///
    /// A pit that retracts during this call cannot rearm until a later update, even
    /// when an occupant remains on it. Deadline arithmetic is preflighted before
    /// mutation, so overflow leaves the clock unchanged.
    pub fn update(
        &mut self,
        at: Tick,
        contacted: bool,
        settings: SpikePitSettings,
    ) -> Result<SpikePitPhase, SpikePitClockError> {
        if let Some(last_updated_at) = self.last_updated_at
            && at < last_updated_at
        {
            return Err(SpikePitClockError::NonMonotonicUpdate {
                at,
                last_updated_at,
            });
        }
        let was_active = !matches!(self.state, SpikePitClockState::Idle);
        let (advanced, retracted) = match self.state {
            SpikePitClockState::Idle => (SpikePitClockState::Idle, false),
            SpikePitClockState::Warning {
                rises_at: _,
                retracts_at,
            } if at >= retracts_at => (SpikePitClockState::Idle, true),
            SpikePitClockState::Warning {
                rises_at,
                retracts_at,
            } if at >= rises_at => (
                SpikePitClockState::Raised {
                    rises_at,
                    retracts_at,
                },
                false,
            ),
            SpikePitClockState::Raised { retracts_at, .. } if at >= retracts_at => {
                (SpikePitClockState::Idle, true)
            }
            current => (current, false),
        };

        if !was_active && contacted && self.last_retraction_update != Some(at) {
            let rises_at =
                at.0.checked_add(settings.warning_ticks())
                    .map(Tick)
                    .ok_or(SpikePitClockError::DeadlineOverflow)?;
            let retracts_at = rises_at
                .0
                .checked_add(settings.raised_ticks())
                .map(Tick)
                .ok_or(SpikePitClockError::DeadlineOverflow)?;
            self.state = SpikePitClockState::Warning {
                rises_at,
                retracts_at,
            };
        } else {
            self.state = advanced;
            if retracted {
                self.last_retraction_update = Some(at);
            }
        }
        self.last_updated_at = Some(at);
        Ok(self.phase())
    }
}

#[must_use]
pub fn position_contacts_spike_pit(position: Position, pit: CellId) -> bool {
    MovementSegment::new(position, position).overlaps_cell(pit)
}

#[must_use]
pub fn segment_contacts_spike_pit(segment: MovementSegment, pit: CellId) -> bool {
    segment.overlaps_cell(pit)
}

#[must_use]
pub fn trace_contacts_spike_pit(trace: &MovementTrace, pit: CellId) -> bool {
    trace.overlaps_cell(pit)
}

#[must_use]
pub fn traces_contact_spike_pit(traces: &[MovementTrace], pit: CellId) -> bool {
    traces
        .iter()
        .any(|trace| trace_contacts_spike_pit(trace, pit))
}

/// Selects a clear reflection orbit from a bounded, RNG-free search.
///
/// Callers pass cells occupied by other overlays, including lairs and every Oil
/// Barrel phase. Four-cell orbits are preferred globally over two-cell
/// center-axis fallbacks. Within each size, nearer-center seeds precede
/// row-major ties. An unavailable member rejects its whole orbit.
#[must_use]
pub fn spike_pit_layout_v1(
    arena: &Arena,
    excluded: impl IntoIterator<Item = CellId>,
) -> Vec<CellId> {
    let excluded = excluded.into_iter().collect::<BTreeSet<_>>();
    let center = CellId::new(arena.width() / 2, arena.height() / 2);
    let mut candidates = (1..arena.height() - 1)
        .flat_map(|row| (1..arena.width() - 1).map(move |column| CellId::new(column, row)))
        .collect::<Vec<_>>();
    candidates.sort_by_key(|cell| {
        (
            cell.column.abs_diff(center.column) + cell.row.abs_diff(center.row),
            *cell,
        )
    });

    for target_size in [MAX_SPIKE_PITS, 2] {
        for &candidate in &candidates {
            let orbit = reflected_orbit(arena, candidate);
            if orbit.len() == target_size
                && orbit
                    .iter()
                    .all(|&cell| placement_is_clear(arena, cell, &excluded))
            {
                return orbit;
            }
        }
    }
    Vec::new()
}

fn reflected_orbit(arena: &Arena, cell: CellId) -> Vec<CellId> {
    let reflected_column = arena.width() - 1 - cell.column;
    let reflected_row = arena.height() - 1 - cell.row;
    BTreeSet::from([
        cell,
        CellId::new(reflected_column, cell.row),
        CellId::new(cell.column, reflected_row),
        CellId::new(reflected_column, reflected_row),
    ])
    .into_iter()
    .collect()
}

fn placement_is_clear(arena: &Arena, cell: CellId, excluded: &BTreeSet<CellId>) -> bool {
    arena.terrain(cell) == Some(Terrain::Floor)
        && !arena.blocks().contains(&cell)
        && !excluded.contains(&cell)
        && potential_spawns(arena)
            .into_iter()
            .all(|spawn| !spawn_reserved(spawn, cell))
}

fn potential_spawns(arena: &Arena) -> [CellId; 4] {
    let right = arena.width() - 2;
    let bottom = arena.height() - 2;
    [
        CellId::new(1, 1),
        CellId::new(right, 1),
        CellId::new(1, bottom),
        CellId::new(right, bottom),
    ]
}

fn spawn_reserved(spawn: CellId, cell: CellId) -> bool {
    spawn.column.abs_diff(cell.column) + spawn.row.abs_diff(cell.row) <= 1
}

#[cfg(test)]
#[path = "spike_pit_tests.rs"]
mod tests;
