use grenadiers_core::{
    game::SeatId,
    protocol::{RoomId, Seat, v3::RoomView},
    room::DepartureReason,
};
use tokio::{
    sync::{mpsc, oneshot, watch},
    task::JoinHandle,
    time::Instant,
};

use super::super::{
    match_runtime::{MatchAction, MatchNotice, MatchRuntimeError},
    types::{BindingKey, DepartureGuard, RoomError, RoomIdentity},
};
use crate::round_seed::RoundSeedSource;

pub(super) const COMMAND_CAPACITY: usize = 32;
pub(in crate::game::rooms) type ShutdownState = bool;
pub(super) type Reply<T> = oneshot::Receiver<Result<T, RoomError>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum ClosingReason {
    Expired,
    Retired,
    ServerStopped,
    InternalFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::game::rooms) enum MatchOwnerNotice {
    Publication(MatchNotice),
    Closing {
        last: MatchNotice,
        reason: ClosingReason,
    },
}

pub(in crate::game::rooms) struct MatchAdmission {
    pub(in crate::game::rooms) room_id: RoomId,
    pub(in crate::game::rooms) round_id: u64,
    pub(in crate::game::rooms) seat: Seat,
    pub(in crate::game::rooms) view: RoomView,
    pub(in crate::game::rooms) notices: watch::Receiver<MatchOwnerNotice>,
    pub(in crate::game::rooms) guard: DepartureGuard,
}

pub(super) enum OwnerCommand {
    Join {
        binding: BindingKey,
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
    pub(super) fn reject(self, error: RoomError) {
        match self {
            Self::Join { reply, .. } => {
                let _ = reply.send(Err(error));
            }
            Self::Action { reply, .. } | Self::Leave { reply, .. } => {
                let _ = reply.send(Err(error));
            }
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
    pub(super) shutdown: watch::Receiver<ShutdownState>,
}

impl PreparedMatchOwner {
    pub(in crate::game::rooms) fn new(
        identity: RoomIdentity,
        created_at: Instant,
        seeds: RoundSeedSource,
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
    pub(super) binding: BindingKey,
    pub(super) departure: oneshot::Receiver<DepartureReason>,
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
        MatchRuntimeError::InvalidPhase | MatchRuntimeError::InvalidInterrupt => {
            RoomError::InvalidPhase
        }
    }
}
