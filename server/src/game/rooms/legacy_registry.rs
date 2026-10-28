use std::{sync::Arc, time::Instant as StdInstant};

use grenadiers_core::{
    protocol::{Counter, RoomCode, RoomId, v13::ResumeToken},
    room::RoomMode,
};
use tokio::time::Instant;

use super::{
    Admission, BindingKey, GameRooms, RoomAction, RoomError, RoomHandle, live::LiveOwner,
    match_owner, registry_lookup::RegistryOperation,
};

impl GameRooms {
    #[cfg(test)]
    pub(crate) async fn create_sequenced(
        self: &Arc<Self>,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        self.create_sequenced_mode(binding, sequence, now, RoomMode::HumanDuel)
            .await
    }

    pub(crate) async fn create_sequenced_mode(
        self: &Arc<Self>,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
        mode: RoomMode,
    ) -> Result<Admission, RoomError> {
        let (handle, owner, id, code) = {
            let mut registry = self.lock();
            if self.is_stopping() {
                return Err(RoomError::RoomNotFound);
            }
            if !registry.operations.take_create(now) {
                return Err(RoomError::RateLimited);
            }
            if registry.rooms.len() >= super::MAX_ROOMS {
                return Err(RoomError::CapacityReached);
            }
            let identity = self.generate_identity(&mut registry, mode)?;
            let id = identity.id.clone();
            let code = identity.code.clone();
            let (owner_handle, owner) = match_owner::PreparedMatchOwner::new_with_planner(
                identity,
                Instant::now(),
                self.round_seed_source.clone(),
                Arc::clone(&self.random_fill),
                self.cpu_planner.clone(),
                self.shutdown(),
            )
            .map_err(|_| RoomError::CapacityReached)?;
            let handle = RoomHandle {
                owner: LiveOwner::Legacy(owner_handle),
            };
            registry.codes.insert(code.clone(), id.clone());
            registry.rooms.insert(id.clone(), handle.clone());
            (handle, owner, id, code)
        };
        let weak = Arc::downgrade(self);
        let cleanup_id = id.clone();
        let cleanup_code = code.clone();
        drop(owner.start(move || {
            if let Some(registry) = weak.upgrade() {
                registry.cleanup(&id, &code);
            }
        }));
        let result = self.admit_legacy(handle, binding, sequence, true).await;
        if result.is_err() {
            self.cleanup(&cleanup_id, &cleanup_code);
        }
        result
    }

    pub(crate) async fn join_sequenced(
        &self,
        code: &RoomCode,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        let handle = self.lookup_code(code, now, RegistryOperation::Join)?;
        self.admit_legacy(handle, binding, sequence, false).await
    }

    #[cfg(test)]
    pub(crate) async fn create(
        self: &Arc<Self>,
        binding: BindingKey,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        self.create_sequenced(binding, Counter::new(1).expect("test sequence"), now)
            .await
    }

    #[cfg(test)]
    pub(crate) async fn join(
        &self,
        code: &RoomCode,
        binding: BindingKey,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        self.join_sequenced(code, binding, Counter::new(1).expect("test sequence"), now)
            .await
    }

    pub(crate) async fn resume(
        &self,
        room_id: RoomId,
        binding: BindingKey,
        sequence: Counter,
        token: ResumeToken,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        let handle = self.lookup_id(&room_id, now, RegistryOperation::Resume)?;
        let owner = handle.owner.legacy().ok_or(RoomError::ResumeRejected)?;
        let mut admission = owner
            .resume(binding, sequence, room_id, token)
            .await
            .unwrap_or(Err(RoomError::ResumeRejected))?;
        admission.notices.borrow_and_update();
        Ok(admission)
    }

    pub(crate) async fn leave(
        &self,
        admission: &Admission,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
    ) -> Result<(), RoomError> {
        let handle = self.handle_for(&admission.room_id)?;
        handle
            .owner
            .legacy()
            .ok_or(RoomError::RoomNotFound)?
            .leave(binding, room_id, round_id)
            .await
            .unwrap_or(Err(RoomError::RoomNotFound))
    }

    pub(crate) async fn action(
        &self,
        admission: &Admission,
        binding: BindingKey,
        room_id: RoomId,
        round_id: u64,
        action: RoomAction,
    ) -> Result<(), RoomError> {
        let handle = self.handle_for(&admission.room_id)?;
        handle
            .owner
            .legacy()
            .ok_or(RoomError::RoomNotFound)?
            .action(binding, room_id, round_id, action)
            .await
            .unwrap_or(Err(RoomError::RoomNotFound))
    }

    async fn admit_legacy(
        &self,
        handle: RoomHandle,
        binding: BindingKey,
        sequence: Counter,
        creator: bool,
    ) -> Result<Admission, RoomError> {
        let owner = handle.owner.legacy().ok_or(RoomError::RoomNotFound)?;
        let reply = if creator {
            owner.admit_creator(binding, sequence)
        } else {
            owner.join_sequenced(binding, sequence)
        };
        let mut admission = reply.await.unwrap_or(Err(RoomError::RoomNotFound))?;
        admission.notices.borrow_and_update();
        Ok(admission)
    }
}
