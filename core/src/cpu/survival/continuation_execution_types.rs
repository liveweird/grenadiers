use std::sync::Arc;

use crate::cpu::routing::RouteFailure;
use crate::cpu::{DecisionId, MemoryKey, MovementIntent};
use crate::game::Tick;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContinuationPublication {
    pub(crate) decision: DecisionId,
    pub(crate) key: MemoryKey,
    pub(crate) sequence: u64,
    pub(crate) tick: Tick,
}

impl ContinuationPublication {
    pub fn new(
        decision: DecisionId,
        key: MemoryKey,
        sequence: u64,
        tick: Tick,
    ) -> Result<Self, RouteFailure> {
        if decision.world != key.world || tick < decision.observation_tick {
            return Err(RouteFailure::IdentityMismatch);
        }
        Ok(Self {
            decision,
            key,
            sequence,
            tick,
        })
    }
}

#[derive(Debug)]
pub(super) struct EpisodeToken;

#[cfg_attr(test, derive(Clone))]
#[derive(Debug)]
pub struct ContinuationCommand {
    pub(super) token: Arc<EpisodeToken>,
    pub(super) movement: MovementIntent,
    pub(super) admission_tick: Tick,
    pub(super) input_index: u8,
}

impl ContinuationCommand {
    pub const fn movement(&self) -> MovementIntent {
        self.movement
    }

    pub const fn admission_tick(&self) -> Tick {
        self.admission_tick
    }

    pub(super) fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.token, &other.token)
            && self.movement == other.movement
            && self.admission_tick == other.admission_tick
            && self.input_index == other.input_index
    }
}

#[derive(Debug)]
pub enum ContinuationStatus {
    Continue,
    Wait,
    Submit(ContinuationCommand),
    RenewalRequired,
    Completed,
    Invalidated,
}
