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
    random_fill: Arc<RandomFill>,
}

impl GameSessions {
    pub(crate) fn new(now: Instant) -> Self {
        Self {
            registry: Mutex::new(Registry::new(now)),
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
            random_fill: Arc::new(random_fill),
        }
    }

    pub(crate) fn issue(&self, now: Instant) -> Result<SessionResponse, SessionIssueError> {
        let mut registry = self
            .registry
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        registry.prune(now);
        registry.rate_limit.refill(now);

        if registry.rate_limit.available < 1.0 {
            return Err(SessionIssueError::RateLimited);
        }
        registry.rate_limit.available -= 1.0;

        if registry.sessions.len() >= MAX_ISSUED_SESSIONS {
            return Err(SessionIssueError::CapacityReached);
        }

        for _ in 0..MAX_COLLISION_RETRIES {
            let token = self.generate_token()?;
            if !registry.sessions.contains_key(&token) {
                registry
                    .sessions
                    .insert(token.clone(), now + SESSION_LIFETIME);
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
    sessions: HashMap<SessionToken, Instant>,
    rate_limit: MintBucket,
}

impl Registry {
    fn new(now: Instant) -> Self {
        Self {
            sessions: HashMap::new(),
            rate_limit: MintBucket {
                available: MINT_BURST,
                updated_at: now,
            },
        }
    }

    fn prune(&mut self, now: Instant) {
        self.sessions.retain(|_, expires_at| *expires_at > now);
    }
}

struct MintBucket {
    available: f64,
    updated_at: Instant,
}

impl MintBucket {
    fn refill(&mut self, now: Instant) {
        let elapsed = now.saturating_duration_since(self.updated_at);
        self.available =
            (self.available + elapsed.as_secs_f64() * MINTS_PER_SECOND).min(MINT_BURST);
        self.updated_at = now;
    }
}

#[cfg(test)]
#[path = "sessions_tests.rs"]
mod tests;
