use rand_chacha::ChaCha20Rng;
use rand_chacha::rand_core::{RngCore, SeedableRng};

pub(super) trait RandomWords {
    fn next_u32(&mut self) -> Option<u32>;
}

pub(super) struct ChaChaWords(ChaCha20Rng);

impl ChaChaWords {
    pub(super) fn seeded(seed: [u8; 32]) -> Self {
        let mut rng = ChaCha20Rng::from_seed(seed);
        rng.set_stream(0);
        rng.set_word_pos(0);
        Self(rng)
    }
}

impl RandomWords for ChaChaWords {
    fn next_u32(&mut self) -> Option<u32> {
        Some(self.0.next_u32())
    }
}
