use crate::cpu::forecast::TickWindow;
use crate::cpu::{MemoryKey, MovementIntent, Observation};
use crate::game::{CellId, Position, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeleporterStepPhase {
    Approach,
    Collect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeleporterStepRequest {
    pub target: CellId,
    pub movement: MovementIntent,
    pub settled_stop_tick: Tick,
}

/// Opaque authority for one movement update followed by a mandatory `Stop`.
/// Deliberately not `Clone`: admission must consume the only authority.
#[derive(Debug)]
pub struct TeleporterStep {
    pub(super) key: MemoryKey,
    pub(super) basis_tick: Tick,
    pub(super) start_window: TickWindow,
    pub(super) safe_until: Tick,
    pub(super) settled_stop_tick: Tick,
    pub(super) target: CellId,
    pub(super) movement: MovementIntent,
    pub(super) phase: TeleporterStepPhase,
    pub(super) initial_position: Position,
    pub(super) clipped_positions: Vec<Position>,
    pub(super) landing_regions: Vec<OutcomeRegion>,
    pub(super) basis: Observation,
}

impl TeleporterStep {
    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }
    #[must_use]
    pub const fn basis_tick(&self) -> Tick {
        self.basis_tick
    }
    #[must_use]
    pub const fn start_window(&self) -> TickWindow {
        self.start_window
    }
    #[must_use]
    pub const fn safe_until(&self) -> Tick {
        self.safe_until
    }
    #[must_use]
    pub const fn settled_stop_tick(&self) -> Tick {
        self.settled_stop_tick
    }
    #[must_use]
    pub const fn target(&self) -> CellId {
        self.target
    }
    #[must_use]
    pub const fn movement(&self) -> MovementIntent {
        self.movement
    }
    #[must_use]
    pub const fn phase(&self) -> TeleporterStepPhase {
        self.phase
    }
    #[must_use]
    pub const fn initial_position(&self) -> Position {
        self.initial_position
    }
    /// Checks a fresh physical stopped pose against every certified outcome.
    #[must_use]
    pub fn allows_stopped_position(&self, position: Position) -> bool {
        self.clipped_positions.contains(&position)
            || self
                .landing_regions
                .iter()
                .any(|region| region.contains(position))
    }
    #[must_use]
    pub const fn basis_observation(&self) -> &Observation {
        &self.basis
    }
}

#[derive(Debug)]
pub(super) struct OutcomeRegion {
    pub(super) min: Position,
    pub(super) max: Position,
}

impl OutcomeRegion {
    fn contains(&self, position: Position) -> bool {
        (self.min.x..=self.max.x).contains(&position.x)
            && (self.min.y..=self.max.y).contains(&position.y)
    }
}
