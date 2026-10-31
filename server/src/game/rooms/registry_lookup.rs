use std::time::Instant as StdInstant;

use grenadiers_core::protocol::{RoomCode, RoomId};

use super::{GameRooms, RoomError, RoomHandle};

impl GameRooms {
    pub(crate) async fn cpu_debug(
        &self,
        room_id: &RoomId,
        token: grenadiers_core::protocol::v14::ResumeToken,
    ) -> Result<crate::cpu_debug::CpuDebugResponse, crate::cpu_debug::CpuDebugQueryError> {
        let owner = {
            let registry = self.lock();
            if self.is_stopping() {
                return Err(crate::cpu_debug::CpuDebugQueryError::Unavailable);
            }
            registry
                .rooms
                .get(room_id)
                .map(|handle| handle.owner.clone())
        }
        .ok_or(crate::cpu_debug::CpuDebugQueryError::Unavailable)?;
        owner.cpu_debug(token).await
    }

    pub(super) fn handle_for(&self, room_id: &RoomId) -> Result<RoomHandle, RoomError> {
        self.lock()
            .rooms
            .get(room_id)
            .cloned()
            .ok_or(RoomError::RoomNotFound)
    }

    pub(super) fn lookup_code(
        &self,
        code: &RoomCode,
        now: StdInstant,
        operation: RegistryOperation,
    ) -> Result<RoomHandle, RoomError> {
        let mut registry = self.lock();
        if self.is_stopping() {
            return Err(RoomError::RoomNotFound);
        }
        if !operation.take(&mut registry.operations, now) {
            return Err(RoomError::RateLimited);
        }
        let id = registry.codes.get(code).ok_or(RoomError::RoomNotFound)?;
        registry
            .rooms
            .get(id)
            .cloned()
            .ok_or(RoomError::RoomNotFound)
    }

    pub(super) fn lookup_id(
        &self,
        room_id: &RoomId,
        now: StdInstant,
        operation: RegistryOperation,
    ) -> Result<RoomHandle, RoomError> {
        let mut registry = self.lock();
        if self.is_stopping() {
            return Err(RoomError::RoomNotFound);
        }
        if !operation.take(&mut registry.operations, now) {
            return Err(RoomError::RateLimited);
        }
        registry
            .rooms
            .get(room_id)
            .cloned()
            .ok_or(RoomError::ResumeRejected)
    }
}

pub(super) enum RegistryOperation {
    Join,
    Resume,
}

impl RegistryOperation {
    fn take(&self, operations: &mut super::OperationBudgets, now: StdInstant) -> bool {
        match self {
            Self::Join => operations.take_join(now),
            Self::Resume => operations.take_resume(now),
        }
    }
}
