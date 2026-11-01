use crate::cpu::forecast::TickWindow;
use crate::cpu::{MemoryKey, MovementIntent, Observation, WorkBudget};
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
    pub(super) nominal_positions: Vec<Position>,
    pub(super) collecting_positions: Vec<Position>,
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

    pub(crate) fn allows_stopped_position_charged(
        &self,
        position: Position,
        work: &mut WorkBudget,
    ) -> Result<bool, super::RouteFailure> {
        let units = self
            .clipped_positions
            .len()
            .checked_add(self.landing_regions.len())
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(super::RouteFailure::Capacity)?;
        work.try_take(units)
            .map_err(|_| super::RouteFailure::WorkExhausted)?;
        Ok(self.allows_stopped_position(position))
    }

    pub fn nominal_makes_progress_toward(
        &self,
        waypoint: CellId,
        work: &mut WorkBudget,
    ) -> Result<bool, super::RouteFailure> {
        let units = u32::try_from(self.nominal_positions.len())
            .map_err(|_| super::RouteFailure::Capacity)?;
        work.try_take(units)
            .map_err(|_| super::RouteFailure::WorkExhausted)?;
        let target = Position::center(waypoint);
        let initial = position_distance(self.initial_position, target);
        Ok(!self.nominal_positions.is_empty()
            && self
                .nominal_positions
                .iter()
                .all(|position| position_distance(*position, target) < initial))
    }

    pub(crate) fn allows_partner_position_charged(
        &self,
        position: Position,
        travel: i64,
        work: &mut WorkBudget,
    ) -> Result<bool, super::RouteFailure> {
        let units = u32::try_from(self.collecting_positions.len())
            .map_err(|_| super::RouteFailure::Capacity)?;
        work.try_take(units)
            .map_err(|_| super::RouteFailure::WorkExhausted)?;
        Ok(self
            .collecting_positions
            .iter()
            .any(|endpoint| position_distance(*endpoint, position) <= travel))
    }

    pub(crate) fn admission_contact_fits(
        &self,
        current: &Observation,
        work: &mut WorkBudget,
    ) -> Result<bool, super::RouteFailure> {
        let contact = super::continuation_contact::ContinuationContact::new(&self.basis, work)
            .map_err(super::teleporter_step_geometry::contact_failure)?;
        contact
            .contains(current, self.safe_until, work)
            .map_err(super::teleporter_step_geometry::contact_failure)
    }

    pub(crate) fn contact_fits_with_expected_marker(
        &self,
        current: &Observation,
        expected_marker: Option<Tick>,
        work: &mut WorkBudget,
    ) -> Result<bool, super::RouteFailure> {
        let contact = super::continuation_contact::ContinuationContact::new(&self.basis, work)
            .map_err(super::teleporter_step_geometry::contact_failure)?;
        contact
            .contains_with_expected_marker(current, self.safe_until, expected_marker, work)
            .map_err(super::teleporter_step_geometry::contact_failure)
    }

    pub(crate) fn pit_layout_fits(
        &self,
        current: &Observation,
        work: &mut WorkBudget,
    ) -> Result<bool, super::RouteFailure> {
        let expected = self.basis.spike_pits();
        let actual = current.spike_pits();
        let expected_len = expected.map_or(0, |pits| pits.pits().len());
        let actual_len = actual.map_or(0, |pits| pits.pits().len());
        let units = expected_len
            .max(actual_len)
            .checked_add(1)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or(super::RouteFailure::Capacity)?;
        work.try_take(units)
            .map_err(|_| super::RouteFailure::WorkExhausted)?;
        Ok(match (expected, actual) {
            (None, None) => true,
            (Some(expected), Some(actual)) => {
                expected.config() == actual.config()
                    && expected.pits().len() == actual.pits().len()
                    && expected
                        .pits()
                        .iter()
                        .zip(actual.pits())
                        .all(|(left, right)| left.cell == right.cell)
            }
            _ => false,
        })
    }
    #[must_use]
    pub const fn basis_observation(&self) -> &Observation {
        &self.basis
    }
}

fn position_distance(left: Position, right: Position) -> i64 {
    (i64::from(left.x) - i64::from(right.x)).abs() + (i64::from(left.y) - i64::from(right.y)).abs()
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
