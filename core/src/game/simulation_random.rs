use std::fmt;

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

pub const SIMULATION_RANDOMNESS_VERSION: u16 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SimulationSeed([u8; 32]);

impl SimulationSeed {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for SimulationSeed {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SimulationSeed(<redacted>)")
    }
}

#[derive(Clone)]
pub(super) struct SimulationRandomness {
    seed: SimulationSeed,
    inner: ChaCha20Rng,
    words_consumed: u64,
}

impl SimulationRandomness {
    pub(super) fn new(seed: SimulationSeed) -> Self {
        Self {
            seed,
            inner: ChaCha20Rng::from_seed(seed.0),
            words_consumed: 0,
        }
    }

    pub(super) fn bounded_index(&mut self, upper: usize) -> Option<usize> {
        let upper = u32::try_from(upper).ok()?;
        debug_assert!(upper > 1);
        sample_bounded(upper, || {
            let word = self.inner.next_u32();
            self.words_consumed = self
                .words_consumed
                .checked_add(1)
                .expect("a simulation cannot consume u64::MAX random words");
            word
        })
        .and_then(|index| usize::try_from(index).ok())
    }

    #[cfg(test)]
    pub(super) const fn words_consumed(&self) -> u64 {
        self.words_consumed
    }
}

fn sample_bounded(upper: u32, mut next: impl FnMut() -> u32) -> Option<u32> {
    let threshold = upper.wrapping_neg() % upper;
    (0..16)
        .map(|_| next())
        .find(|&word| word >= threshold)
        .map(|word| word % upper)
}

impl PartialEq for SimulationRandomness {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed && self.words_consumed == other.words_consumed
    }
}
impl Eq for SimulationRandomness {}

impl fmt::Debug for SimulationRandomness {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SimulationRandomness")
            .field("seed", &"<redacted>")
            .field("state", &"<redacted>")
            .field("words_consumed", &self.words_consumed)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_rejection_consumes_at_most_sixteen_words_without_fallback_bias() {
        let mut draws = 0;
        assert_eq!(
            sample_bounded(3, || {
                draws += 1;
                0
            }),
            None
        );
        assert_eq!(draws, 16);

        let mut words = [0, 0, 4].into_iter();
        assert_eq!(sample_bounded(3, || words.next().unwrap()), Some(1));
    }
}
