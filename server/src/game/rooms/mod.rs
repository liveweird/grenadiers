use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant as StdInstant,
};

use grenadiers_core::protocol::{Counter, RoomCode, RoomId, v9::ResumeToken};
use tokio::{
    sync::{Notify, watch},
    time::Instant,
};

mod input;
mod match_owner;
mod match_runtime;
#[cfg(test)]
mod measurement;
mod profile;
mod rate_limit;
mod replay;
mod resume;
mod types;
use crate::round_seed::RoundSeedSource;
pub(crate) use match_owner::{
    ClosingReason, MatchAdmission as Admission, MatchOwnerNotice as RoomNotice,
};
pub(crate) use match_runtime::MatchAction as RoomAction;
pub(crate) use match_runtime::MatchNotice;
use rate_limit::Bucket;
use types::RoomIdentity;
pub(crate) use types::{BindingKey, RoomError};

const MAX_ROOMS: usize = 32;
const OPERATION_BURST: f64 = 32.0;
const OPERATIONS_PER_SECOND: f64 = 8.0;
const COLLISION_RETRIES: usize = 3;
const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub(super) type RandomFill = dyn Fn(&mut [u8]) -> Result<(), ()> + Send + Sync;

pub(crate) struct GameRooms {
    registry: Mutex<Registry>,
    random_fill: Arc<RandomFill>,
    round_seed_source: RoundSeedSource,
    profile: profile::RoomProfile,
    shutdown: watch::Sender<bool>,
    empty: Notify,
}

#[derive(Clone)]
struct RoomHandle {
    owner: match_owner::MatchOwnerHandle,
}

struct Registry {
    rooms: HashMap<RoomId, RoomHandle>,
    codes: HashMap<RoomCode, RoomId>,
    prefix: Option<u64>,
    next_serial: u64,
    operations: Bucket,
}

impl GameRooms {
    pub(crate) fn new(now: StdInstant, round_seed_source: RoundSeedSource) -> Arc<Self> {
        let (shutdown, _) = watch::channel(false);
        Arc::new(Self {
            registry: Mutex::new(Registry {
                rooms: HashMap::new(),
                codes: HashMap::new(),
                prefix: None,
                next_serial: 1,
                operations: Bucket::new(now),
            }),
            random_fill: Arc::new(|bytes| getrandom::fill(bytes).map_err(|_| ())),
            round_seed_source,
            profile: profile::RoomProfile::ExpandedV5,
            shutdown,
            empty: Notify::new(),
        })
    }

    #[cfg(test)]
    pub(crate) fn with_random_fill(
        now: StdInstant,
        fill: impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + 'static,
    ) -> Arc<Self> {
        let (shutdown, _) = watch::channel(false);
        Self {
            registry: Mutex::new(Registry {
                rooms: HashMap::new(),
                codes: HashMap::new(),
                prefix: None,
                next_serial: 1,
                operations: Bucket::new(now),
            }),
            random_fill: Arc::new(fill),
            round_seed_source: RoundSeedSource::from_optional_path(None)
                .expect("OS entropy source construction cannot fail"),
            profile: profile::RoomProfile::FixedClassicV1,
            shutdown,
            empty: Notify::new(),
        }
        .into()
    }

    #[cfg(test)]
    pub(crate) fn with_random_fill_and_round_seed(
        now: StdInstant,
        fill: impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + 'static,
        round_seed_source: RoundSeedSource,
    ) -> Arc<Self> {
        let (shutdown, _) = watch::channel(false);
        Self {
            registry: Mutex::new(Registry {
                rooms: HashMap::new(),
                codes: HashMap::new(),
                prefix: None,
                next_serial: 1,
                operations: Bucket::new(now),
            }),
            random_fill: Arc::new(fill),
            round_seed_source,
            profile: profile::RoomProfile::ExpandedV5,
            shutdown,
            empty: Notify::new(),
        }
        .into()
    }

    pub(crate) async fn create_sequenced(
        self: &Arc<Self>,
        binding: BindingKey,
        sequence: Counter,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        let (handle, owner, id, code) = {
            let mut registry = self.lock();
            if self.is_stopping() {
                return Err(RoomError::RoomNotFound);
            }
            if !registry.operations.take(now) {
                return Err(RoomError::RateLimited);
            }
            if registry.rooms.len() >= MAX_ROOMS {
                return Err(RoomError::CapacityReached);
            }
            let identity = self.generate_identity(&mut registry)?;
            let cleanup_id = identity.id.clone();
            let cleanup_code = identity.code.clone();
            let (owner_handle, owner) = match_owner::PreparedMatchOwner::new(
                identity.clone(),
                Instant::now(),
                self.round_seed_source.clone(),
                Arc::clone(&self.random_fill),
                self.shutdown(),
            )
            .map_err(|_| RoomError::CapacityReached)?;
            let handle = RoomHandle {
                owner: owner_handle,
            };
            registry
                .codes
                .insert(identity.code.clone(), identity.id.clone());
            registry.rooms.insert(identity.id.clone(), handle.clone());
            (handle, owner, cleanup_id, cleanup_code)
        };
        let registry = Arc::downgrade(self);
        let cleanup_id = id.clone();
        let cleanup_code = code.clone();
        drop(owner.start(move || {
            if let Some(registry) = registry.upgrade() {
                registry.cleanup(&id, &code);
            }
        }));
        let result = self.admit(handle, binding, sequence).await;
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
        let handle = {
            let mut registry = self.lock();
            if self.is_stopping() {
                return Err(RoomError::RoomNotFound);
            }
            if !registry.operations.take(now) {
                return Err(RoomError::RateLimited);
            }
            let id = registry.codes.get(code).ok_or(RoomError::RoomNotFound)?;
            registry
                .rooms
                .get(id)
                .cloned()
                .ok_or(RoomError::RoomNotFound)?
        };
        self.admit(handle, binding, sequence).await
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
        let handle = {
            let mut registry = self.lock();
            if self.is_stopping() {
                return Err(RoomError::RoomNotFound);
            }
            if !registry.operations.take(now) {
                return Err(RoomError::RateLimited);
            }
            registry
                .rooms
                .get(&room_id)
                .cloned()
                .ok_or(RoomError::ResumeRejected)?
        };
        let mut admission = handle
            .owner
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
        let handle = {
            let registry = self.lock();
            registry
                .rooms
                .get(&admission.room_id)
                .cloned()
                .ok_or(RoomError::RoomNotFound)?
        };
        handle
            .owner
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
        let handle = {
            let registry = self.lock();
            registry
                .rooms
                .get(&admission.room_id)
                .cloned()
                .ok_or(RoomError::RoomNotFound)?
        };
        handle
            .owner
            .action(binding, room_id, round_id, action)
            .await
            .unwrap_or(Err(RoomError::RoomNotFound))
    }

    async fn admit(
        &self,
        handle: RoomHandle,
        binding: BindingKey,
        sequence: Counter,
    ) -> Result<Admission, RoomError> {
        let mut admission = handle
            .owner
            .join_sequenced(binding, sequence)
            .await
            .unwrap_or(Err(RoomError::RoomNotFound))?;
        admission.notices.borrow_and_update();
        Ok(admission)
    }

    fn generate_identity(&self, registry: &mut Registry) -> Result<RoomIdentity, RoomError> {
        let prefix = match registry.prefix {
            Some(prefix) => prefix,
            None => {
                let mut bytes = [0; 8];
                (self.random_fill)(&mut bytes).map_err(|_| RoomError::CapacityReached)?;
                let prefix = u64::from_be_bytes(bytes);
                registry.prefix = Some(prefix);
                prefix
            }
        };
        let serial = registry.next_serial;
        let next_serial = serial.checked_add(1).ok_or(RoomError::CapacityReached)?;
        for _ in 0..COLLISION_RETRIES {
            let code = self.generate_code()?;
            if !registry.codes.contains_key(&code) {
                let id = RoomId::new(&format!("{prefix:016x}{serial:016x}"))
                    .map_err(|_| RoomError::CapacityReached)?;
                registry.next_serial = next_serial;
                return Ok(RoomIdentity {
                    id,
                    code,
                    profile: self.profile,
                });
            }
        }
        Err(RoomError::CapacityReached)
    }

    fn generate_code(&self) -> Result<RoomCode, RoomError> {
        let mut bytes = [0; 5];
        (self.random_fill)(&mut bytes).map_err(|_| RoomError::CapacityReached)?;
        let bits = u64::from_be_bytes([0, 0, 0, bytes[0], bytes[1], bytes[2], bytes[3], bytes[4]]);
        let code = (0..8)
            .map(|shift| CODE_ALPHABET[((bits >> (35 - shift * 5)) & 31) as usize] as char)
            .collect::<String>();
        RoomCode::new(&code).map_err(|_| RoomError::CapacityReached)
    }

    pub(super) fn cleanup(&self, id: &RoomId, code: &RoomCode) {
        let mut registry = self.lock();
        if registry.codes.get(code) == Some(id) && registry.rooms.contains_key(id) {
            registry.codes.remove(code);
            registry.rooms.remove(id);
            if registry.rooms.is_empty() {
                self.empty.notify_waiters();
            }
        }
    }

    pub(crate) fn shutdown(&self) -> watch::Receiver<bool> {
        self.shutdown.subscribe()
    }

    pub(crate) fn is_stopping(&self) -> bool {
        *self.shutdown.borrow()
    }

    pub(crate) fn stop(&self) {
        self.shutdown.send_replace(true);
    }

    pub(crate) async fn wait_empty(&self) {
        loop {
            let notified = self.empty.notified();
            if self.lock().rooms.is_empty() {
                return;
            }
            notified.await;
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.registry
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

#[cfg(test)]
#[path = "rooms_tests.rs"]
mod tests;
