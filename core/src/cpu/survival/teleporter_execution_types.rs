use std::sync::Arc;

use crate::cpu::policy::UtilityMemory;
use crate::cpu::routing::TeleporterStep;
use crate::cpu::{DecisionId, MemoryKey, MovementIntent};
use crate::game::Tick;

#[derive(Debug)]
pub(super) struct EpisodeToken;

#[derive(Debug)]
pub struct TeleporterStepCommand {
    pub(super) token: Arc<EpisodeToken>,
    pub(super) key: MemoryKey,
    pub(super) command_epoch: u64,
    pub(super) movement: MovementIntent,
    pub(super) admission_tick: Tick,
    pub(super) index: u8,
}

impl TeleporterStepCommand {
    #[must_use]
    pub const fn movement(&self) -> MovementIntent {
        self.movement
    }
    #[must_use]
    pub const fn admission_tick(&self) -> Tick {
        self.admission_tick
    }
    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }
    #[must_use]
    pub const fn command_epoch(&self) -> u64 {
        self.command_epoch
    }

    pub(super) fn matches(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.token, &other.token)
            && self.key == other.key
            && self.command_epoch == other.command_epoch
            && self.movement == other.movement
            && self.admission_tick == other.admission_tick
            && self.index == other.index
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeleporterOutcome {
    pub swapped: bool,
    pub consumed: bool,
}

#[derive(Debug)]
pub enum TeleporterStepStatus {
    Moving,
    WaitForStoppedObservation,
    Submit(TeleporterStepCommand),
    Completed(TeleporterOutcome),
    Invalidated,
}

pub struct PreparedTeleporterStep {
    pub(super) token: Arc<EpisodeToken>,
    pub(super) step: Box<TeleporterStep>,
    pub(super) next_memory: UtilityMemory,
    pub(super) decision: DecisionId,
    pub(super) admission_tick: Tick,
    pub(super) command_issued: bool,
    pub(super) roster_v48: bool,
}

pub struct CommittedTeleporterStep {
    pub(super) token: Arc<EpisodeToken>,
    pub(super) step: Box<TeleporterStep>,
    pub(super) next_memory: UtilityMemory,
    pub(super) decision: DecisionId,
    pub(super) admission_tick: Tick,
    pub(super) stop_issued: bool,
    pub(super) stop_recorded: bool,
    pub(super) completed: Option<TeleporterOutcome>,
    pub(super) revoked: bool,
    pub(super) roster_v48: bool,
}

pub(super) fn command(
    token: &Arc<EpisodeToken>,
    key: MemoryKey,
    command_epoch: u64,
    movement: MovementIntent,
    admission_tick: Tick,
    index: u8,
) -> TeleporterStepCommand {
    TeleporterStepCommand {
        token: Arc::clone(token),
        key,
        command_epoch,
        movement,
        admission_tick,
        index,
    }
}
