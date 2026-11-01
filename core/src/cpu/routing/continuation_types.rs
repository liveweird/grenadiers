use std::collections::BTreeSet;

use crate::cpu::forecast::ForecastEvolution;
use crate::cpu::{MemoryKey, MovementIntent, Observation};
use crate::game::{Direction, EntityId, MovementSegment, Tick};

use super::continuation_contact::ContinuationContact;
use super::motion::MotionSnapshot;
use super::pulse::PulseOriginGrace;
use super::{RouteAssurance, SpikePitGuard};

pub(super) const MAX_CONTINUATION_LEGS: usize = 8;
pub(super) const MAX_CONTINUATION_MOVES: u16 = 96;
pub(super) const STOP_BRANCH_UPDATES: u64 = 13;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NominalLeg {
    Move { direction: Direction, updates: u8 },
    Stop { updates: u8 },
}

impl NominalLeg {
    pub(super) const fn movement(self) -> MovementIntent {
        match self {
            Self::Move { direction, .. } => MovementIntent::Direction(direction),
            Self::Stop { .. } => MovementIntent::Stop,
        }
    }

    pub(super) const fn updates(self) -> u8 {
        match self {
            Self::Move { updates, .. } | Self::Stop { updates } => updates,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContinuationRequest {
    pub(crate) first_update: Tick,
    pub(crate) settled_stop_tick: Tick,
    pub(crate) survive_until: Tick,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ContinuationCheckpoint {
    pub(super) tick: Tick,
    pub(super) movement: MovementIntent,
    pub(super) motion: MotionSnapshot,
    pub(super) segments: Vec<MovementSegment>,
    pub(super) origin_active: bool,
    pub(super) pit_guard: SpikePitGuard,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct StopBranch {
    pub(super) publication: Tick,
    pub(super) checkpoints: Vec<ContinuationCheckpoint>,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContinuationCheckpointView {
    pub(crate) tick: Tick,
    pub(crate) movement: MovementIntent,
    pub(crate) position: crate::game::Position,
    pub(crate) heading: Direction,
    pub(crate) residual: u16,
    pub(crate) pending: Option<(Direction, i32)>,
    pub(crate) reversed: bool,
}

// Runtime continuation authority is linear. Tests may fork immutable proof data
// to exercise independent failure branches, but production cannot duplicate it.
#[cfg_attr(test, derive(Clone))]
#[allow(dead_code)] // Inactive guidance; executor integration is a later package.
pub struct ContinuationCertificate {
    key: MemoryKey,
    basis: Observation,
    basis_tick: Tick,
    first_update: Tick,
    survive_until: Tick,
    origin: PulseOriginGrace,
    admission_checkpoint: ContinuationCheckpoint,
    legs: Vec<NominalLeg>,
    checkpoints: Vec<ContinuationCheckpoint>,
    stop_branches: Vec<StopBranch>,
    assurance: RouteAssurance,
    assumptions: BTreeSet<EntityId>,
    evolution: ForecastEvolution,
    contact: ContinuationContact,
}

pub(super) struct ContinuationParts {
    pub(super) key: MemoryKey,
    pub(super) basis: Observation,
    pub(super) basis_tick: Tick,
    pub(super) request: ContinuationRequest,
    pub(super) origin: PulseOriginGrace,
    pub(super) admission_checkpoint: ContinuationCheckpoint,
    pub(super) legs: Vec<NominalLeg>,
    pub(super) checkpoints: Vec<ContinuationCheckpoint>,
    pub(super) stop_branches: Vec<StopBranch>,
    pub(super) assumptions: BTreeSet<EntityId>,
    pub(super) evolution: ForecastEvolution,
    pub(super) contact: ContinuationContact,
}

#[allow(dead_code)] // Accessors are consumed by the later bounded grant package.
impl ContinuationCertificate {
    pub(super) fn from_parts(parts: ContinuationParts) -> Option<Self> {
        let assurance = if parts.assumptions.is_empty() {
            RouteAssurance::Certified
        } else {
            RouteAssurance::ConditionalStationary {
                bombs: parts.assumptions.iter().copied().collect(),
            }
        };
        (!parts.checkpoints.is_empty()).then_some(Self {
            key: parts.key,
            basis: parts.basis,
            basis_tick: parts.basis_tick,
            first_update: parts.request.first_update,
            survive_until: parts.request.survive_until,
            origin: parts.origin,
            admission_checkpoint: parts.admission_checkpoint,
            legs: parts.legs,
            checkpoints: parts.checkpoints,
            stop_branches: parts.stop_branches,
            assurance,
            assumptions: parts.assumptions,
            evolution: parts.evolution,
            contact: parts.contact,
        })
    }

    pub(crate) const fn key(&self) -> MemoryKey {
        self.key
    }

    pub(crate) const fn basis(&self) -> &Observation {
        &self.basis
    }

    pub(crate) const fn basis_tick(&self) -> Tick {
        self.basis_tick
    }

    pub(crate) const fn first_update(&self) -> Tick {
        self.first_update
    }

    pub(crate) const fn survive_until(&self) -> Tick {
        self.survive_until
    }

    #[cfg(test)]
    pub(crate) fn set_survive_until_for_test(&mut self, survive_until: Tick) {
        self.survive_until = survive_until;
    }

    pub(super) const fn origin(&self) -> PulseOriginGrace {
        self.origin
    }

    pub(super) const fn admission_checkpoint(&self) -> &ContinuationCheckpoint {
        &self.admission_checkpoint
    }

    #[cfg(test)]
    pub(crate) const fn origin_for_test(&self) -> (crate::game::CellId, EntityId) {
        (self.origin.cell, self.origin.bomb_id)
    }

    #[cfg(test)]
    pub(crate) fn admission_checkpoint_for_test(&self) -> ContinuationCheckpointView {
        ContinuationCheckpointView {
            tick: self.admission_checkpoint.tick,
            movement: self.admission_checkpoint.movement,
            position: self.admission_checkpoint.motion.position,
            heading: self.admission_checkpoint.motion.heading,
            residual: self.admission_checkpoint.motion.residual,
            pending: self.admission_checkpoint.motion.pending,
            reversed: self.admission_checkpoint.motion.reversed,
        }
    }

    pub(super) fn legs(&self) -> &[NominalLeg] {
        &self.legs
    }

    #[cfg(test)]
    pub(crate) fn legs_for_test(&self) -> &[NominalLeg] {
        &self.legs
    }

    pub(super) fn checkpoints(&self) -> &[ContinuationCheckpoint] {
        &self.checkpoints
    }

    pub(super) fn stop_branches(&self) -> &[StopBranch] {
        &self.stop_branches
    }

    pub(super) const fn assurance(&self) -> &RouteAssurance {
        &self.assurance
    }

    pub(super) fn assumptions(&self) -> &BTreeSet<EntityId> {
        &self.assumptions
    }

    pub(crate) const fn evolution(&self) -> &ForecastEvolution {
        &self.evolution
    }

    pub(super) const fn contact(&self) -> &ContinuationContact {
        &self.contact
    }

    #[cfg(test)]
    pub(crate) fn checkpoints_for_test(&self) -> Vec<ContinuationCheckpointView> {
        self.checkpoints
            .iter()
            .map(|checkpoint| ContinuationCheckpointView {
                tick: checkpoint.tick,
                movement: checkpoint.movement,
                position: checkpoint.motion.position,
                heading: checkpoint.motion.heading,
                residual: checkpoint.motion.residual,
                pending: checkpoint.motion.pending,
                reversed: checkpoint.motion.reversed,
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn assumptions_for_test(&self) -> &BTreeSet<EntityId> {
        &self.assumptions
    }

    #[cfg(test)]
    pub(crate) fn stop_branch_shape_for_test(&self) -> Vec<(Tick, usize, Tick)> {
        self.stop_branches
            .iter()
            .filter_map(|branch| {
                Some((
                    branch.publication,
                    branch.checkpoints.len(),
                    branch.checkpoints.last()?.tick,
                ))
            })
            .collect()
    }
}
