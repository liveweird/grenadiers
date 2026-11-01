use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant as StdInstant,
};

use grenadiers_core::protocol::{RoomCode, RoomId};
use tokio::sync::{Notify, watch};

mod closing;
mod cpu;
#[cfg(test)]
mod cpu_debug_test_support;
mod input;
#[cfg(test)]
mod legacy_registry;
#[cfg(test)]
mod legacy_v14;
mod live;
mod live_registry;
#[cfg(test)]
mod match_owner;
#[cfg(test)]
mod match_runtime;
#[cfg(test)]
mod measurement;
#[cfg(test)]
mod profile;
mod rate_limit;
mod registry_identity;
mod registry_lookup;
#[cfg(test)]
mod replay;
#[cfg(test)]
#[allow(dead_code)]
mod replay_v10;
#[cfg(test)]
#[allow(dead_code)] // Private accepted-operation recording facade; production recording stays off.
mod replay_v11;
#[cfg(test)]
#[allow(dead_code)] // Private accepted-input replay facade; production export remains disabled.
mod replay_v12;
#[cfg(test)]
#[allow(dead_code)] // Private accepted-input executor; production export remains disabled.
mod replay_v13;
#[cfg(test)]
#[allow(dead_code)] // Private accepted-input executor; production export remains disabled.
mod replay_v14;
#[cfg(test)]
#[allow(dead_code)] // Exact accepted-input policy binding; production export remains disabled.
mod replay_v15;
#[cfg(test)]
#[allow(dead_code)] // Replay4 is exercised through its test-only recording facade.
mod replay_v4;
#[cfg(test)]
#[allow(dead_code)]
mod replay_v6;
#[cfg(test)]
#[allow(dead_code)]
mod replay_v7;
#[cfg(test)]
#[allow(dead_code)]
mod replay_v8;
#[cfg(test)]
#[allow(dead_code)]
mod replay_v9;
mod resume;
mod roster_owner;
mod roster_runtime;
mod roster_seed;
mod types;
use crate::round_seed::RoundSeedSource;
pub(crate) use closing::ClosingReason;
pub(crate) use live::{LiveAdmission, LiveDelivery, LiveMemberEvent};
pub(crate) use live_registry::LairSettingsAction;
#[cfg(test)]
pub(crate) use match_owner::{MatchAdmission as Admission, MatchOwnerNotice as RoomNotice};
#[cfg(test)]
pub(crate) use match_runtime::MatchAction as RoomAction;
#[cfg(test)]
pub(crate) use match_runtime::MatchNotice;
use rate_limit::OperationBudgets;
pub(crate) use roster_runtime::RosterAction;
#[cfg(test)]
use types::RoomIdentity;
pub(crate) use types::{BindingKey, RoomError};

const MAX_ROOMS: usize = 32;
const COLLISION_RETRIES: usize = 3;
const CODE_ALPHABET: &[u8; 32] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";

pub(super) type RandomFill = dyn Fn(&mut [u8]) -> Result<(), ()> + Send + Sync;

pub(crate) struct GameRooms {
    registry: Mutex<Registry>,
    random_fill: Arc<RandomFill>,
    round_seed_source: RoundSeedSource,
    cpu_planner: cpu::CpuPlanner,
    #[cfg(test)]
    profile: profile::RoomProfile,
    shutdown: watch::Sender<bool>,
    empty: Notify,
}

#[derive(Clone)]
struct RoomHandle {
    owner: live::LiveOwner,
}

struct Registry {
    rooms: HashMap<RoomId, RoomHandle>,
    codes: HashMap<RoomCode, RoomId>,
    prefix: Option<u64>,
    next_serial: u64,
    operations: OperationBudgets,
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
                operations: OperationBudgets::new(now),
            }),
            random_fill: Arc::new(|bytes| getrandom::fill(bytes).map_err(|_| ())),
            round_seed_source,
            cpu_planner: cpu::CpuPlanner::new().expect("CPU planner workers must start"),
            #[cfg(test)]
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
                operations: OperationBudgets::new(now),
            }),
            random_fill: Arc::new(fill),
            round_seed_source: RoundSeedSource::from_optional_path(None)
                .expect("OS entropy source construction cannot fail"),
            cpu_planner: cpu::CpuPlanner::new().expect("CPU planner workers must start"),
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
                operations: OperationBudgets::new(now),
            }),
            random_fill: Arc::new(fill),
            round_seed_source,
            cpu_planner: cpu::CpuPlanner::new().expect("CPU planner workers must start"),
            profile: profile::RoomProfile::ExpandedV5,
            shutdown,
            empty: Notify::new(),
        }
        .into()
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
#[path = "live_registry_tests.rs"]
mod live_registry_tests;
#[cfg(test)]
#[path = "rooms_tests.rs"]
mod tests;
