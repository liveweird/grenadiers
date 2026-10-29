use std::sync::Arc;

use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, RoomId, v14},
    room::DepartureReason,
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    time::Instant,
};

use super::super::{
    ClosingReason, RandomFill,
    cpu::CpuController,
    resume::{BindingFence, ResumeCredential},
    roster_runtime::{RosterAction, RosterNotice, RosterRuntime, RosterRuntimeError},
    types::{BindingKey, DepartureGuard, RoomError},
};

pub(super) const COMMAND_CAPACITY: usize = 32;
pub(super) type Reply<T> = oneshot::Receiver<Result<T, RoomError>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RosterOwnerNotice {
    Publication(RosterNotice),
    Closing {
        last: RosterNotice,
        reason: ClosingReason,
    },
}

pub(crate) struct RosterAdmission {
    pub(crate) room_id: RoomId,
    #[cfg(test)]
    pub(crate) round_id: u64,
    pub(crate) seat: SeatId,
    #[cfg(test)]
    pub(crate) room_code: grenadiers_core::protocol::RoomCode,
    #[cfg(test)]
    pub(crate) revision: u32,
    #[cfg(test)]
    pub(crate) resume_token: v14::ResumeToken,
    pub(crate) personal_frame: Arc<str>,
    pub(crate) notices: watch::Receiver<RosterOwnerNotice>,
    pub(crate) guard: DepartureGuard,
    pub(crate) replaced: oneshot::Receiver<()>,
}

pub(super) enum OwnerCommand {
    Join {
        creator: bool,
        binding: BindingKey,
        sequence: Counter,
        reply: oneshot::Sender<Result<RosterAdmission, RoomError>>,
    },
    Resume {
        binding: BindingKey,
        sequence: Counter,
        room_id: RoomId,
        token: v14::ResumeToken,
        reply: oneshot::Sender<Result<RosterAdmission, RoomError>>,
    },
    Action {
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: RosterAction,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
    EnvironmentSettings {
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: super::super::live_registry::EnvironmentSettingsAction,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
    Leave {
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        reply: oneshot::Sender<Result<(), RoomError>>,
    },
}

impl OwnerCommand {
    pub(super) const fn is_admission(&self) -> bool {
        matches!(self, Self::Join { .. } | Self::Resume { .. })
    }

    pub(super) fn reject(self, error: RoomError) {
        match self {
            Self::Join { reply, .. } | Self::Resume { reply, .. } => {
                let _ = reply.send(Err(error));
            }
            Self::Action { reply, .. }
            | Self::EnvironmentSettings { reply, .. }
            | Self::Leave { reply, .. } => {
                let _ = reply.send(Err(error));
            }
        }
    }

    pub(super) fn reject_unavailable(self) {
        if matches!(self, Self::Resume { .. }) {
            self.reject(RoomError::ResumeRejected);
        } else {
            self.reject(RoomError::RoomNotFound);
        }
    }
}

#[derive(Clone)]
pub(in crate::game::rooms) struct RosterOwnerHandle {
    pub(super) commands: mpsc::Sender<OwnerCommand>,
}

pub(in crate::game::rooms) struct PreparedRosterOwner {
    pub(super) created_at: Instant,
    pub(super) receiver: mpsc::Receiver<OwnerCommand>,
    pub(super) notices: watch::Sender<RosterOwnerNotice>,
    pub(super) initial: RosterNotice,
    pub(super) runtime: RosterRuntime,
    pub(super) random_fill: Arc<RandomFill>,
    pub(super) cpus: Vec<CpuController>,
    pub(super) shutdown: watch::Receiver<bool>,
}

pub(super) struct SeatSlot {
    pub(super) credential: ResumeCredential,
    pub(super) live: Option<LiveBinding>,
}

pub(super) struct LiveBinding {
    pub(super) fence: BindingFence,
    pub(super) departure: oneshot::Receiver<DepartureReason>,
    pub(super) replaced: Option<oneshot::Sender<()>>,
    pub(super) pending: Option<PendingDeparture>,
}

pub(super) struct PendingDeparture {
    pub(super) binding: BindingKey,
    pub(super) reason: DepartureReason,
    pub(super) reply: Option<oneshot::Sender<Result<(), RoomError>>>,
}

pub(super) fn map_runtime_error(error: RosterRuntimeError) -> RoomError {
    match error {
        RosterRuntimeError::RateLimited => RoomError::RateLimited,
        RosterRuntimeError::DuplicateAction => RoomError::DuplicateAction,
        RosterRuntimeError::StaleSettings => RoomError::StaleSettings,
        RosterRuntimeError::SettingsLocked => RoomError::SettingsLocked,
        RosterRuntimeError::StaleEpoch | RosterRuntimeError::VacantSeat => RoomError::StaleRoom,
        RosterRuntimeError::InvalidPhase => RoomError::InvalidPhase,
        RosterRuntimeError::UnsupportedConfiguration
        | RosterRuntimeError::CapacityReached
        | RosterRuntimeError::Publication
        | RosterRuntimeError::SeedUnavailable => RoomError::CapacityReached,
    }
}
