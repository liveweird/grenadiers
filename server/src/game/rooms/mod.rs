use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant as StdInstant,
};

use grenadiers_core::protocol::{RoomCode, RoomId};
use tokio::{
    sync::{mpsc, oneshot},
    time::Instant,
};

mod input;
mod owner;
mod profile;
mod publication;
mod runtime;
mod types;

use crate::round_seed::RoundSeedSource;
pub(crate) use input::RoomAction;
use types::RoomIdentity;
pub(crate) use types::{Admission, BindingKey, RoomError, RoomNotice};

const MAX_ROOMS: usize = 32;
const OPERATION_BURST: f64 = 32.0;
const OPERATIONS_PER_SECOND: f64 = 8.0;
const COLLISION_RETRIES: usize = 3;
const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

type RandomFill = dyn Fn(&mut [u8]) -> Result<(), ()> + Send + Sync;

pub(crate) struct GameRooms {
    registry: Mutex<Registry>,
    random_fill: Arc<RandomFill>,
    round_seed_source: RoundSeedSource,
    profile: profile::RoomProfile,
}

#[derive(Clone)]
struct RoomHandle {
    commands: mpsc::Sender<owner::RoomCommand>,
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
            profile: profile::RoomProfile::GeneratedClassicV2,
        })
    }

    #[cfg(test)]
    pub(crate) fn with_random_fill(
        now: StdInstant,
        fill: impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + 'static,
    ) -> Arc<Self> {
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
        }
        .into()
    }

    #[cfg(test)]
    pub(crate) fn with_random_fill_and_round_seed(
        now: StdInstant,
        fill: impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + 'static,
        round_seed_source: RoundSeedSource,
    ) -> Arc<Self> {
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
            profile: profile::RoomProfile::GeneratedClassicV2,
        }
        .into()
    }

    pub(crate) async fn create(
        self: &Arc<Self>,
        binding: BindingKey,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        let (handle, owner) = {
            let mut registry = self.lock();
            if !registry.operations.take(now) {
                return Err(RoomError::RateLimited);
            }
            if registry.rooms.len() >= MAX_ROOMS {
                return Err(RoomError::CapacityReached);
            }
            let identity = self.generate_identity(&mut registry)?;
            let (commands, owner) = owner::PreparedOwner::new(
                identity.clone(),
                Instant::now(),
                self.round_seed_source.clone(),
            );
            let handle = RoomHandle { commands };
            registry
                .codes
                .insert(identity.code.clone(), identity.id.clone());
            registry.rooms.insert(identity.id.clone(), handle.clone());
            (handle, owner)
        };
        drop(owner.start(Arc::downgrade(self)));
        self.admit(handle, binding).await
    }

    pub(crate) async fn join(
        &self,
        code: &RoomCode,
        binding: BindingKey,
        now: StdInstant,
    ) -> Result<Admission, RoomError> {
        let handle = {
            let mut registry = self.lock();
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
        self.admit(handle, binding).await
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
        let (reply, response) = oneshot::channel();
        handle
            .commands
            .try_send(owner::RoomCommand::Leave {
                binding,
                room_id,
                round_id,
                reply,
            })
            .map_err(map_send_error)?;
        response.await.unwrap_or(Err(RoomError::RoomNotFound))
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
        let (reply, response) = oneshot::channel();
        handle
            .commands
            .try_send(owner::RoomCommand::Action {
                binding,
                room_id,
                round_id,
                action,
                reply,
            })
            .map_err(map_send_error)?;
        response.await.unwrap_or(Err(RoomError::RoomNotFound))
    }

    async fn admit(&self, handle: RoomHandle, binding: BindingKey) -> Result<Admission, RoomError> {
        let (reply, response) = oneshot::channel();
        handle
            .commands
            .try_send(owner::RoomCommand::Join { binding, reply })
            .map_err(map_send_error)?;
        response.await.unwrap_or(Err(RoomError::RoomNotFound))
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
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Registry> {
        self.registry
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}

fn map_send_error<T>(error: mpsc::error::TrySendError<T>) -> RoomError {
    match error {
        mpsc::error::TrySendError::Full(_) => RoomError::RateLimited,
        mpsc::error::TrySendError::Closed(_) => RoomError::RoomNotFound,
    }
}

struct Bucket {
    available: f64,
    updated_at: StdInstant,
}

impl Bucket {
    fn new(now: StdInstant) -> Self {
        Self {
            available: OPERATION_BURST,
            updated_at: now,
        }
    }

    fn take(&mut self, now: StdInstant) -> bool {
        let elapsed = now.saturating_duration_since(self.updated_at);
        self.available =
            (self.available + elapsed.as_secs_f64() * OPERATIONS_PER_SECOND).min(OPERATION_BURST);
        self.updated_at = self.updated_at.max(now);
        if self.available < 1.0 {
            false
        } else {
            self.available -= 1.0;
            true
        }
    }
}

#[cfg(test)]
#[path = "rooms_tests.rs"]
mod tests;
