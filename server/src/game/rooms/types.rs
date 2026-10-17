use std::sync::Arc;

use grenadiers_core::protocol::{RoomCode, RoomId, RoomView, Seat};
use tokio::sync::{oneshot, watch};

use super::super::sessions::ConnectionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BindingKey {
    pub(crate) connection_id: ConnectionId,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum RoomNotice {
    View(RoomView),
    Snapshot(Arc<str>),
    Expired,
    Retired,
}

pub(crate) struct DepartureGuard {
    sender: Option<oneshot::Sender<grenadiers_core::room::DepartureReason>>,
}

impl DepartureGuard {
    pub(super) fn new(sender: oneshot::Sender<grenadiers_core::room::DepartureReason>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    pub(crate) fn disarm(&mut self) {
        self.sender = None;
    }
}

impl Drop for DepartureGuard {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(grenadiers_core::room::DepartureReason::TransportLoss);
        }
    }
}

pub(crate) struct Admission {
    pub(crate) room_id: RoomId,
    pub(crate) round_id: u64,
    pub(crate) seat: Seat,
    pub(crate) view: RoomView,
    pub(crate) notices: watch::Receiver<RoomNotice>,
    pub(crate) guard: DepartureGuard,
}

#[derive(Clone)]
pub(super) struct RoomIdentity {
    pub(super) id: RoomId,
    pub(super) code: RoomCode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoomError {
    RateLimited,
    CapacityReached,
    RoomNotFound,
    RoomFull,
    StaleRoom,
    InvalidPhase,
    DuplicateAction,
}
