//! Bounded hazard forecasting from recipient-visible facts.

mod engine;
mod history;
mod intervals;
mod model;
mod stationary;
mod uncertainty;

use crate::game::{BlastKind, CellId, EntityId, Tick};

use super::MemoryKey;

pub use engine::forecast;
pub use history::{HazardMemoryError, ObservedHazardMemory};

pub const MAX_FORECAST_HORIZON_TICKS: u16 = 840;
pub const MAX_INTERVALS_PER_CELL: u8 = 32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TickWindow {
    pub start: Tick,
    pub end: Tick,
}

impl TickWindow {
    #[must_use]
    pub const fn contains(self, tick: Tick) -> bool {
        self.start.0 <= tick.0 && tick.0 < self.end.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HazardKind {
    Lethal,
    Blocking,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UncertaintyReason {
    BombMotion(EntityId),
    Trigger(EntityId),
    Pause(EntityId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UncertainWindow {
    pub window: TickWindow,
    pub kind: HazardKind,
    pub reason: UncertaintyReason,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellForecast {
    pub lethal: Vec<TickWindow>,
    pub blocking: Vec<TickWindow>,
    pub uncertainty: Vec<UncertainWindow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HazardForecast {
    key: MemoryKey,
    basis_tick: Tick,
    horizon_end: Tick,
    cells: Vec<CellForecast>,
    stationary_assumptions: Vec<EntityId>,
}

impl HazardForecast {
    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }

    #[must_use]
    pub const fn basis_tick(&self) -> Tick {
        self.basis_tick
    }

    #[must_use]
    pub const fn horizon_end(&self) -> Tick {
        self.horizon_end
    }

    #[must_use]
    pub fn cell(&self, cell: CellId) -> Option<&CellForecast> {
        let index = usize::from(cell.row)
            .checked_mul(usize::from(crate::game::arena::ARENA_WIDTH))?
            .checked_add(usize::from(cell.column))?;
        self.cells.get(index)
    }

    #[must_use]
    pub fn cells(&self) -> &[CellForecast] {
        &self.cells
    }

    #[must_use]
    pub fn stationary_assumptions(&self) -> &[EntityId] {
        &self.stationary_assumptions
    }

    pub(super) fn from_parts(
        key: MemoryKey,
        basis_tick: Tick,
        horizon_end: Tick,
        cells: Vec<CellForecast>,
        stationary_assumptions: Vec<EntityId>,
        limits: ForecastLimits,
    ) -> Result<Self, ForecastFailure> {
        if horizon_end.0 <= basis_tick.0
            || cells.len() != crate::game::arena::ARENA_CELL_COUNT
            || stationary_assumptions.len() > crate::cpu::MAX_OBSERVED_BOMBS
            || cells.iter().any(|cell| {
                cell.lethal.len() > usize::from(limits.max_intervals_per_cell)
                    || cell.blocking.len() > usize::from(limits.max_intervals_per_cell)
                    || cell.uncertainty.len() > usize::from(limits.max_intervals_per_cell)
            })
        {
            return Err(ForecastFailure::IntervalCapacity);
        }
        Ok(Self {
            key,
            basis_tick,
            horizon_end,
            cells,
            stationary_assumptions,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ForecastLimits {
    pub horizon_ticks: u16,
    pub max_intervals_per_cell: u8,
}

impl Default for ForecastLimits {
    fn default() -> Self {
        Self {
            horizon_ticks: MAX_FORECAST_HORIZON_TICKS,
            max_intervals_per_cell: MAX_INTERVALS_PER_CELL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HypotheticalPlacement {
    pub cell: CellId,
    pub range: u16,
    pub blast: BlastKind,
    pub detonates_at: Tick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForecastFailure {
    InvalidLimits,
    WrongWorld,
    BombKnowledgeUnavailable,
    InvalidObservedHazards,
    UnsupportedRules,
    InvalidHypothetical,
    ClockOverflow,
    WorkExhausted,
    IntervalCapacity,
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

#[cfg(test)]
#[path = "phase_tests.rs"]
mod phase_tests;

#[cfg(test)]
#[path = "safety_review_tests.rs"]
mod safety_review_tests;
