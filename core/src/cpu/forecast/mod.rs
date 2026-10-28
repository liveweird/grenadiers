//! Bounded hazard forecasting from recipient-visible facts.

mod engine;
mod evolution;
mod history;
mod intervals;
mod model;
mod punch;
mod stationary;
mod uncertainty;

use crate::game::{BlastKind, CellId, EntityId, Tick};
use crate::room::BoardSize;

use super::MemoryKey;

pub use engine::forecast;
pub use evolution::ForecastEvolution;
pub use history::{HazardMemoryError, ObservedHazardMemory};
pub(crate) use punch::forecast_punch_branch;

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
    board: BoardSize,
    basis_tick: Tick,
    horizon_end: Tick,
    cells: Vec<CellForecast>,
    stationary_assumptions: Vec<EntityId>,
    evolution: ForecastEvolution,
    hypothetical_id: Option<EntityId>,
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
    pub const fn dimensions(&self) -> (u16, u16) {
        self.board.dimensions()
    }

    #[must_use]
    pub fn cell(&self, cell: CellId) -> Option<&CellForecast> {
        let (width, height) = self.dimensions();
        if cell.column >= width || cell.row >= height {
            return None;
        }
        let index = usize::from(cell.row)
            .checked_mul(usize::from(width))?
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

    #[must_use]
    pub const fn evolution(&self) -> &ForecastEvolution {
        &self.evolution
    }

    #[must_use]
    pub(crate) const fn hypothetical_id(&self) -> Option<EntityId> {
        self.hypothetical_id
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn from_parts(
        key: MemoryKey,
        board: BoardSize,
        basis_tick: Tick,
        horizon_end: Tick,
        cells: Vec<CellForecast>,
        stationary_assumptions: Vec<EntityId>,
        evolution: ForecastEvolution,
        hypothetical_id: Option<EntityId>,
        limits: ForecastLimits,
    ) -> Result<Self, ForecastFailure> {
        if horizon_end.0 <= basis_tick.0
            || cells.len() != board.cell_count()
            || stationary_assumptions.len() > crate::cpu::MAX_MULTIPLAYER_OBSERVED_BOMBS
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
            board,
            basis_tick,
            horizon_end,
            cells,
            stationary_assumptions,
            evolution,
            hypothetical_id,
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
    pub placed_at: Tick,
    pub owner: EntityId,
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

#[cfg(test)]
#[path = "evolution_tests.rs"]
mod evolution_tests;

#[cfg(test)]
#[path = "temporal_placement_tests.rs"]
mod temporal_placement_tests;

#[cfg(test)]
#[path = "punch_tests.rs"]
mod punch_tests;

#[cfg(test)]
mod multiplayer_capacity_tests;

#[cfg(test)]
mod dimension_tests;
