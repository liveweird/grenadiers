use grenadiers_core::protocol::{RoomCode, RoomId};
use tokio::sync::oneshot;

use super::super::sessions::ConnectionId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct BindingKey {
    pub(crate) connection_id: ConnectionId,
    pub(crate) generation: u64,
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

#[derive(Clone)]
pub(super) struct RoomIdentity {
    pub(super) id: RoomId,
    pub(super) code: RoomCode,
    pub(super) profile: super::profile::RoomProfile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoomError {
    RateLimited,
    CapacityReached,
    RoomNotFound,
    RoomFull,
    StaleRoom,
    StaleSettings,
    SettingsLocked,
    InvalidPhase,
    DuplicateAction,
    ResumeRejected,
}
