use crate::cpu::forecast::{ForecastEvolution, ForecastFailure};
use crate::cpu::{BombKnowledge, MemoryKey, Observation, ObservedBomb, ObservedBombTrigger};
use crate::game::{BlastKind, CellId, EntityId, Tick};

use super::{RouteCertificate, RouteFailure, RouteLimits};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PunchRequest {
    pub execution_tick: Tick,
    pub settled_stop_tick: Tick,
    pub limits: RouteLimits,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunchOutcome {
    Awaiting,
    Unchanged,
    Relocated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PunchFailure {
    InvalidRequest,
    Unsupported,
    Forecast(ForecastFailure),
    Route(RouteFailure),
    SceneMismatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BombSignature {
    pub(super) id: EntityId,
    pub(super) placed_at: Tick,
    pub(super) source: CellId,
    pub(super) owner: EntityId,
    pub(super) range: u16,
    pub(super) detonates_at: Tick,
}

impl BombSignature {
    pub(super) fn matches(self, bomb: &ObservedBomb, cell: CellId, tick: Tick) -> bool {
        bomb.id == self.id
            && bomb.placed_at == self.placed_at
            && bomb.cell == cell
            && bomb.owner == self.owner
            && bomb.range == self.range
            && bomb.blast == BlastKind::Standard
            && matches!(
                bomb.trigger,
                ObservedBombTrigger::Timed {
                    remaining_ticks,
                    paused: false,
                } if remaining_ticks > 0
                    && tick.0.checked_add(u64::from(remaining_ticks))
                        == Some(self.detonates_at.0)
            )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PunchCertificate {
    key: MemoryKey,
    route: RouteCertificate,
    execution_tick: Tick,
    bomb: BombSignature,
    landing: CellId,
    hypothetical_id: EntityId,
    unchanged: ForecastEvolution,
    relocated: ForecastEvolution,
    relocated_basis: Observation,
}

impl PunchCertificate {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        route: RouteCertificate,
        execution_tick: Tick,
        bomb: BombSignature,
        landing: CellId,
        hypothetical_id: EntityId,
        unchanged: ForecastEvolution,
        relocated: ForecastEvolution,
        relocated_basis: Observation,
    ) -> Self {
        Self {
            key: route.key(),
            route,
            execution_tick,
            bomb,
            landing,
            hypothetical_id,
            unchanged,
            relocated,
            relocated_basis,
        }
    }

    #[must_use]
    pub const fn route(&self) -> &RouteCertificate {
        &self.route
    }

    #[must_use]
    pub const fn execution_tick(&self) -> Tick {
        self.execution_tick
    }

    pub fn confirm(
        &self,
        basis: &Observation,
        current: &Observation,
    ) -> Result<PunchOutcome, PunchFailure> {
        let BombKnowledge::Visible(bombs) = &current.parts().bombs else {
            return Err(PunchFailure::SceneMismatch);
        };
        let bomb = bombs
            .iter()
            .find(|candidate| candidate.id == self.bomb.id)
            .ok_or(PunchFailure::SceneMismatch)?;
        let outcome = if current.parts().tick < self.execution_tick {
            self.bomb
                .matches(bomb, self.bomb.source, current.parts().tick)
                .then_some(PunchOutcome::Awaiting)
        } else if self
            .bomb
            .matches(bomb, self.bomb.source, current.parts().tick)
        {
            Some(PunchOutcome::Unchanged)
        } else if self.bomb.matches(bomb, self.landing, current.parts().tick) {
            Some(PunchOutcome::Relocated)
        } else {
            None
        }
        .ok_or(PunchFailure::SceneMismatch)?;
        self.matches_scene(basis, current, outcome)
            .then_some(outcome)
            .ok_or(PunchFailure::SceneMismatch)
    }

    #[must_use]
    pub fn matches_scene(
        &self,
        basis: &Observation,
        current: &Observation,
        outcome: PunchOutcome,
    ) -> bool {
        if self.key.world != basis.parts().world
            || self.key.cpu_seat != basis.parts().recipient
            || !basis_bomb_matches(basis, self.bomb)
        {
            return false;
        }
        match outcome {
            PunchOutcome::Awaiting => {
                current.parts().tick < self.execution_tick
                    && current_bomb_matches(current, self.bomb, self.bomb.source)
                    && self.unchanged.matches_scene(basis, current)
            }
            PunchOutcome::Unchanged => {
                current.parts().tick >= self.execution_tick
                    && (self
                        .unchanged
                        .expected_live_bombs(self.bomb.owner, current.parts().tick)
                        == Some(0)
                        || current_bomb_matches(current, self.bomb, self.bomb.source))
                    && self.unchanged.matches_scene(basis, current)
            }
            PunchOutcome::Relocated => {
                current.parts().tick >= self.execution_tick
                    && (self
                        .relocated
                        .hypothetical_removed(self.hypothetical_id, current.parts().tick)
                        || current_bomb_matches(current, self.bomb, self.landing))
                    && self.relocated.matches_scene_with_hypothetical(
                        &self.relocated_basis,
                        current,
                        self.hypothetical_id,
                        self.bomb.id,
                    )
            }
        }
    }

    #[must_use]
    pub fn expected_live_bombs(
        &self,
        owner: EntityId,
        tick: Tick,
        outcome: PunchOutcome,
    ) -> Option<u16> {
        match outcome {
            PunchOutcome::Awaiting | PunchOutcome::Unchanged => {
                self.unchanged.expected_live_bombs(owner, tick)
            }
            PunchOutcome::Relocated => self.relocated.expected_live_bombs(owner, tick),
        }
    }

    #[must_use]
    pub fn can_admit(
        &self,
        basis: &Observation,
        current: &Observation,
        admission_tick: Tick,
    ) -> bool {
        super::punch::can_admit(self, basis, current, admission_tick)
    }

    pub(super) const fn bomb(&self) -> BombSignature {
        self.bomb
    }

    pub(super) const fn landing(&self) -> CellId {
        self.landing
    }
}

fn basis_bomb_matches(basis: &Observation, signature: BombSignature) -> bool {
    matches!(&basis.parts().bombs, BombKnowledge::Visible(bombs)
        if bombs.len() == 1
            && signature.matches(&bombs[0], signature.source, basis.parts().tick))
}

fn current_bomb_matches(current: &Observation, signature: BombSignature, cell: CellId) -> bool {
    matches!(&current.parts().bombs, BombKnowledge::Visible(bombs)
        if bombs.iter().any(|bomb| signature.matches(bomb, cell, current.parts().tick)))
}

impl From<ForecastFailure> for PunchFailure {
    fn from(value: ForecastFailure) -> Self {
        Self::Forecast(value)
    }
}

impl From<RouteFailure> for PunchFailure {
    fn from(value: RouteFailure) -> Self {
        Self::Route(value)
    }
}
