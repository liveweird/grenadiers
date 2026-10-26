use std::fmt;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

use super::MemoryKey;

/// An RNG stream owned exclusively by one CPU controller.
#[derive(Clone)]
pub struct CpuRng {
    key: MemoryKey,
    inner: ChaCha20Rng,
    draws: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RandomError {
    EmptyRange,
    DrawCountOverflow,
}

impl CpuRng {
    #[must_use]
    pub fn from_seed(key: MemoryKey, seed: [u8; 32]) -> Self {
        Self {
            key,
            inner: ChaCha20Rng::from_seed(seed),
            draws: 0,
        }
    }

    #[must_use]
    pub const fn key(&self) -> MemoryKey {
        self.key
    }

    /// Starts a fresh stream for a new physical world.
    pub fn reset_for(&mut self, key: MemoryKey, seed: [u8; 32]) {
        self.key = key;
        self.inner = ChaCha20Rng::from_seed(seed);
        self.draws = 0;
    }

    /// Samples `0..upper` using exactly one random word.
    ///
    /// Multiply-high mapping has a small, fixed bias in exchange for fixed work
    /// and draw consumption; it is suitable for policy tie-breaking, not secrets.
    pub fn bounded_u32(&mut self, upper: u32) -> Result<u32, RandomError> {
        if upper == 0 {
            return Err(RandomError::EmptyRange);
        }
        let draws = self
            .draws
            .checked_add(1)
            .ok_or(RandomError::DrawCountOverflow)?;
        let word = self.inner.next_u32();
        self.draws = draws;
        Ok(((u64::from(word) * u64::from(upper)) >> 32) as u32)
    }

    #[must_use]
    pub const fn draws(&self) -> u64 {
        self.draws
    }
}

impl fmt::Debug for CpuRng {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CpuRng")
            .field("key", &self.key)
            .field("seed", &"<redacted>")
            .field("state", &"<redacted>")
            .field("draws", &self.draws)
            .finish()
    }
}
