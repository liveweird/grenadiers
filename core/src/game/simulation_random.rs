use std::{fmt, num::NonZeroU64};

use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

pub const SIMULATION_RANDOMNESS_VERSION: u16 = 1;
/// Pinned private-root to physical-round seed derivation used by replay format 4.
pub const SIMULATION_SEED_DERIVATION_VERSION: u16 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct SimulationSeed([u8; 32]);

impl SimulationSeed {
    #[must_use]
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Derive an independent seed for a physical state epoch without advancing this root.
    ///
    /// Version 1 takes the first 32 bytes of ChaCha20 using the root as key and the
    /// nonzero epoch as the 64-bit stream. Callers must obtain this root independently
    /// of arena and CPU roots; canceled countdowns need not derive any seed.
    #[must_use]
    pub fn for_round(self, epoch: NonZeroU64) -> Self {
        let mut rng = ChaCha20Rng::from_seed(self.0);
        rng.set_stream(epoch.get());
        let mut bytes = [0; 32];
        rng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    pub(crate) const fn bytes(self) -> [u8; 32] {
        self.0
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

    pub(super) const fn seed(&self) -> SimulationSeed {
        self.seed
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
    fn round_seed_derivation_is_independent_of_request_order_and_root() {
        let root = SimulationSeed::new([7; 32]);
        let epoch = |value| NonZeroU64::new(value).unwrap();
        let first = root.for_round(epoch(1));
        let later = root.for_round(epoch(u64::MAX));
        assert_ne!(first, root);
        assert_ne!(first, later);
        assert_ne!(first, SimulationSeed::new([8; 32]).for_round(epoch(1)));
        assert_eq!(first, root.for_round(epoch(1)));
        assert_eq!(later, root.for_round(epoch(u64::MAX)));
        assert!(!format!("{first:?}").contains("["));
    }

    #[test]
    fn round_seed_derivation_v1_matches_independent_chacha20_vectors() {
        // OpenSSL ChaCha20: key=07 repeated, IV=zero 64-bit counter then LE epoch.
        assert_eq!(SIMULATION_SEED_DERIVATION_VERSION, 1);
        for (epoch, expected) in [
            (
                1_u64,
                [
                    0x6d, 0x7c, 0x73, 0x09, 0x99, 0xba, 0x65, 0xef, 0xdf, 0x3c, 0x6a, 0x34, 0x19,
                    0x8c, 0xf1, 0xa5, 0x3a, 0x8f, 0x96, 0xd8, 0xe7, 0xad, 0x9a, 0x51, 0x43, 0xc9,
                    0xc5, 0x70, 0x2b, 0x4a, 0xf9, 0xb8,
                ],
            ),
            (
                2_u64,
                [
                    0xc5, 0x19, 0x84, 0xa7, 0xe2, 0x06, 0xde, 0xc9, 0xc0, 0x12, 0x12, 0xfa, 0xd2,
                    0xdb, 0x11, 0x0e, 0x05, 0x27, 0x32, 0x84, 0x56, 0x18, 0x82, 0x7a, 0x09, 0x32,
                    0x3d, 0xd1, 0x01, 0x54, 0x86, 0x40,
                ],
            ),
            (
                18446744073709551615_u64,
                [
                    0xf3, 0x5e, 0x71, 0x21, 0x8a, 0x55, 0xe5, 0xc4, 0x36, 0x60, 0xb8, 0x11, 0xc7,
                    0x90, 0xda, 0x1e, 0x8e, 0xc8, 0xae, 0xf2, 0x15, 0xd0, 0xf4, 0x4d, 0xa9, 0xf8,
                    0x4d, 0x9e, 0x41, 0x0d, 0x70, 0x4a,
                ],
            ),
        ] {
            let seed = SimulationSeed::new([7; 32]).for_round(NonZeroU64::new(epoch).unwrap());
            assert_eq!(seed.0, expected);
        }
    }

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
