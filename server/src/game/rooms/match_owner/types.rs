use grenadiers_core::{
    game::SeatId,
    protocol::{Counter, RoomId, Seat, v9::ResumeToken},
    room::DepartureReason,
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
    time::Instant,
};

use super::super::{
    match_runtime::{MatchAction, MatchNotice, MatchRuntimeError},
    resume::{BindingFence, ResumeCredential},
    types::{BindingKey, DepartureGuard, RoomError, RoomIdentity},
};
use crate::{game::rooms::RandomFill, round_seed::RoundSeedSource};
use std::sync::Arc;

pub(super) const COMMAND_CAPACITY: usize = 32;
pub(in crate::game::rooms) type ShutdownState = bool;
pub(super) type Reply<T> = oneshot::Receiver<Result<T, RoomError>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ClosingReason {
    Expired,
    Retired,
    ServerStopped,
    InternalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum MatchOwnerNotice {
    Publication(MatchNotice),
    Closing {
        last: MatchNotice,
        reason: ClosingReason,
    },
}

pub(crate) struct MatchAdmission {
    pub(crate) room_id: RoomId,
    pub(crate) round_id: u64,
    pub(crate) seat: Seat,
    pub(crate) room: grenadiers_core::protocol::v9::RoomView,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) resume_token: ResumeToken,
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) last_bomb_action_id: Option<Counter>,
    pub(crate) personal_frame: Arc<str>,
    pub(crate) notices: watch::Receiver<MatchOwnerNotice>,
    pub(crate) guard: DepartureGuard,
    pub(crate) replaced: oneshot::Receiver<()>,
}

pub(super) enum OwnerCommand {
    Join {
        binding: BindingKey,
        sequence: Counter,
        reply: oneshot::Sender<Result<MatchAdmission, RoomError>>,
    },
    Resume {
        binding: BindingKey,
        sequence: Counter,
        room_id: RoomId,
        token: ResumeToken,
        reply: oneshot::Sender<Result<MatchAdmission, RoomError>>,
    },
    Action {
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: MatchAction,
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
            Self::Action { reply, .. } | Self::Leave { reply, .. } => {
                let _ = reply.send(Err(error));
            }
        }
    }

    pub(super) fn reject_unavailable(self) {
        match self {
            Self::Resume { reply, .. } => {
                let _ = reply.send(Err(RoomError::ResumeRejected));
            }
            command => command.reject(RoomError::RoomNotFound),
        }
    }
}

#[derive(Clone)]
pub(in crate::game::rooms) struct MatchOwnerHandle {
    pub(super) commands: mpsc::Sender<OwnerCommand>,
}

pub(in crate::game::rooms) struct PreparedMatchOwner {
    pub(super) created_at: Instant,
    pub(super) receiver: mpsc::Receiver<OwnerCommand>,
    pub(super) notices: watch::Sender<MatchOwnerNotice>,
    pub(super) initial: MatchNotice,
    pub(super) runtime: super::super::match_runtime::MatchRuntime,
    pub(super) random_fill: Arc<RandomFill>,
    pub(super) shutdown: watch::Receiver<ShutdownState>,
}

impl PreparedMatchOwner {
    pub(in crate::game::rooms) fn new(
        identity: RoomIdentity,
        created_at: Instant,
        seeds: RoundSeedSource,
        random_fill: Arc<RandomFill>,
        shutdown: watch::Receiver<ShutdownState>,
    ) -> Result<(MatchOwnerHandle, Self), MatchRuntimeError> {
        let mut runtime =
            super::super::match_runtime::MatchRuntime::new(identity.clone(), seeds.clone());
        let initial = runtime.initial_notice()?;
        let (notices, receiver) = watch::channel(MatchOwnerNotice::Publication(initial.clone()));
        drop(receiver);
        let (commands, command_receiver) = mpsc::channel(COMMAND_CAPACITY);
        Ok((
            MatchOwnerHandle { commands },
            Self {
                created_at,
                receiver: command_receiver,
                notices,
                initial,
                runtime,
                random_fill,
                shutdown,
            },
        ))
    }

    pub(in crate::game::rooms) fn start(
        self,
        on_exit: impl FnOnce() + Send + 'static,
    ) -> JoinHandle<()> {
        let guard = CleanupGuard(Some(Box::new(on_exit)));
        tokio::spawn(async move {
            let _guard = guard;
            super::runner::run(self).await;
        })
    }
}

struct CleanupGuard(Option<Box<dyn FnOnce() + Send>>);

impl Drop for CleanupGuard {
    fn drop(&mut self) {
        if let Some(cleanup) = self.0.take() {
            cleanup();
        }
    }
}

pub(super) struct PendingDeparture {
    pub(super) binding: BindingKey,
    pub(super) reason: DepartureReason,
    pub(super) reply: Option<oneshot::Sender<Result<(), RoomError>>>,
}

pub(super) struct SeatSlot {
    pub(super) credential: Option<ResumeCredential>,
    pub(super) live: Option<LiveBinding>,
}

pub(super) struct LiveBinding {
    pub(super) fence: BindingFence,
    pub(super) departure: oneshot::Receiver<DepartureReason>,
    pub(super) replaced: Option<oneshot::Sender<()>>,
    pub(super) pending: Option<PendingDeparture>,
}

pub(super) const fn wire_seat(seat: SeatId) -> Seat {
    match seat {
        SeatId::One => Seat::One,
        SeatId::Two => Seat::Two,
    }
}

pub(super) fn map_runtime_error(error: MatchRuntimeError) -> RoomError {
    match error {
        MatchRuntimeError::RateLimited => RoomError::RateLimited,
        MatchRuntimeError::DuplicateAction => RoomError::DuplicateAction,
        MatchRuntimeError::CapacityReached | MatchRuntimeError::Publication => {
            RoomError::CapacityReached
        }
        MatchRuntimeError::StaleEpoch | MatchRuntimeError::VacantSeat => RoomError::StaleRoom,
        MatchRuntimeError::StaleSettings => RoomError::StaleSettings,
        MatchRuntimeError::SettingsLocked => RoomError::SettingsLocked,
        MatchRuntimeError::InvalidPhase | MatchRuntimeError::InvalidInterrupt => {
            RoomError::InvalidPhase
        }
    }
}
