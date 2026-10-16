use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use grenadiers_core::protocol::SessionToken;
use serde::Serialize;

const MAX_ISSUED_SESSIONS: usize = 128;
const SESSION_LIFETIME: Duration = Duration::from_secs(300);
const MINT_BURST: f64 = 32.0;
const MINTS_PER_SECOND: f64 = 8.0;
const TOKEN_BYTES: usize = 32;
const MAX_COLLISION_RETRIES: usize = 3;

type RandomFill = dyn Fn(&mut [u8]) -> Result<(), ()> + Send + Sync;

#[derive(Serialize)]
pub(crate) struct SessionResponse {
    token: SessionToken,
}

impl SessionResponse {
    fn new(token: SessionToken) -> Self {
        Self { token }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionIssueError {
    RateLimited,
    CapacityReached,
    CreationUnavailable,
}

pub(crate) struct GameSessions {
    registry: Mutex<Registry>,
    upgrade_limit: Mutex<MintBucket>,
    random_fill: Arc<RandomFill>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConnectionId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SessionClaimError {
    AuthenticationFailed,
    ConnectionInUse,
    CapacityReached,
}

pub(crate) struct SessionLease {
    sessions: Arc<GameSessions>,
    token: SessionToken,
    connection_id: ConnectionId,
}

impl GameSessions {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            registry: Mutex::new(Registry::new(now)),
            upgrade_limit: Mutex::new(MintBucket::new(now, MINT_BURST)),
            random_fill: Arc::new(|bytes| getrandom::fill(bytes).map_err(|_| ())),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_random_fill(
        now: Instant,
        random_fill: impl Fn(&mut [u8]) -> Result<(), ()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            registry: Mutex::new(Registry::new(now)),
            upgrade_limit: Mutex::new(MintBucket::new(now, MINT_BURST)),
            random_fill: Arc::new(random_fill),
        }
    }

    pub(crate) fn allow_upgrade(&self, now: Instant) -> bool {
        self.upgrade_limit
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .take(now, MINTS_PER_SECOND)
    }

    pub(crate) fn claim(
        self: &Arc<Self>,
        token: &SessionToken,
        now: Instant,
    ) -> Result<SessionLease, SessionClaimError> {
        let mut registry = self
            .registry
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let now = registry.effective_now(now);
        registry.prune(now);
        match registry.sessions.get(token) {
            None => return Err(SessionClaimError::AuthenticationFailed),
            Some(SessionState::Claimed { .. }) => return Err(SessionClaimError::ConnectionInUse),
            Some(SessionState::Issued { .. }) => {}
        }
        let connection_id = ConnectionId(registry.next_connection_id);
        registry.next_connection_id = registry
            .next_connection_id
            .checked_add(1)
            .ok_or(SessionClaimError::CapacityReached)?;
        registry
            .sessions
            .insert(token.clone(), SessionState::Claimed { connection_id });
        Ok(SessionLease {
            sessions: Arc::clone(self),
            token: token.clone(),
            connection_id,
        })
    }

    pub(crate) fn issue(&self, now: Instant) -> Result<SessionResponse, SessionIssueError> {
        let mut registry = self
            .registry
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let now = registry.effective_now(now);
        registry.prune(now);
        if !registry.rate_limit.take(now, MINTS_PER_SECOND) {
            return Err(SessionIssueError::RateLimited);
        }

        if registry.sessions.len() >= MAX_ISSUED_SESSIONS {
            return Err(SessionIssueError::CapacityReached);
        }

        for _ in 0..MAX_COLLISION_RETRIES {
            let token = self.generate_token()?;
            if !registry.sessions.contains_key(&token) {
                registry.sessions.insert(
                    token.clone(),
                    SessionState::Issued {
                        expires_at: now + SESSION_LIFETIME,
                    },
                );
                return Ok(SessionResponse::new(token));
            }
        }
        Err(SessionIssueError::CreationUnavailable)
    }

    fn generate_token(&self) -> Result<SessionToken, SessionIssueError> {
        let mut bytes = [0_u8; TOKEN_BYTES];
        (self.random_fill)(&mut bytes).map_err(|()| SessionIssueError::CreationUnavailable)?;
        let mut encoded = String::with_capacity(TOKEN_BYTES * 2);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in bytes {
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        SessionToken::new(&encoded).map_err(|_| SessionIssueError::CreationUnavailable)
    }
}

struct Registry {
    sessions: HashMap<SessionToken, SessionState>,
    rate_limit: MintBucket,
    latest_now: Instant,
    next_connection_id: u64,
}

enum SessionState {
    Issued { expires_at: Instant },
    Claimed { connection_id: ConnectionId },
}

impl Registry {
    fn new(now: Instant) -> Self {
        Self {
            sessions: HashMap::new(),
            rate_limit: MintBucket::new(now, MINT_BURST),
            latest_now: now,
            next_connection_id: 1,
        }
    }

    // Requests capture time before taking the mutex. Clamp that value once so
    // every registry operation observes the same nondecreasing timeline.
    fn effective_now(&mut self, captured_now: Instant) -> Instant {
        self.latest_now = self.latest_now.max(captured_now);
        self.latest_now
    }

    fn prune(&mut self, now: Instant) {
        self.sessions.retain(|_, state| match state {
            SessionState::Issued { expires_at } => *expires_at > now,
            SessionState::Claimed { .. } => true,
        });
    }
}

struct MintBucket {
    available: f64,
    updated_at: Instant,
}

impl MintBucket {
    fn new(now: Instant, available: f64) -> Self {
        Self {
            available,
            updated_at: now,
        }
    }

    fn take(&mut self, now: Instant, per_second: f64) -> bool {
        self.refill(now, per_second);
        if self.available < 1.0 {
            false
        } else {
            self.available -= 1.0;
            true
        }
    }

    fn refill(&mut self, now: Instant, per_second: f64) {
        let Some(elapsed) = now.checked_duration_since(self.updated_at) else {
            return;
        };
        self.available = (self.available + elapsed.as_secs_f64() * per_second).min(MINT_BURST);
        self.updated_at = now;
    }
}

impl Drop for SessionLease {
    fn drop(&mut self) {
        let mut registry = self
            .sessions
            .registry
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if matches!(
            registry.sessions.get(&self.token),
            Some(SessionState::Claimed { connection_id }) if *connection_id == self.connection_id
        ) {
            registry.sessions.remove(&self.token);
        }
    }
}

#[cfg(test)]
#[path = "sessions_tests.rs"]
mod tests;
