//! Reproducible, per-session randomness. This is not a cryptographic secret.
use rand::{RngExt, SeedableRng};
use rand_chacha::ChaCha20Rng;

pub const ALGORITHM_VERSION: u32 = 1;
// ChaCha20 has a 64-bit block counter and 16 four-byte words per block.
pub const WORD_POSITION_END: u128 = 1u128 << 68;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RngState {
    pub algorithm_version: u32,
    pub seed: [u8; 32],
    pub stream: u64,
    pub word_position: u128,
}

#[derive(Clone)]
pub struct SessionRng {
    seed: [u8; 32],
    generator: ChaCha20Rng,
}

impl SessionRng {
    pub fn random() -> Self {
        Self::from_seed(rand::rng().random())
    }

    pub fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            seed,
            generator: ChaCha20Rng::from_seed(seed),
        }
    }

    pub fn state(&self) -> RngState {
        RngState {
            algorithm_version: ALGORITHM_VERSION,
            seed: self.seed,
            stream: self.generator.get_stream(),
            word_position: self.generator.get_word_pos(),
        }
    }

    pub fn from_state(state: RngState) -> Result<Self, &'static str> {
        if state.algorithm_version != ALGORITHM_VERSION {
            return Err("unsupported RNG algorithm version");
        }
        // rand_chacha's setter silently discards bits above 68; reject
        // noncanonical positions before accepting untrusted save state.
        if state.word_position >= WORD_POSITION_END {
            return Err("RNG word position out of range");
        }
        let mut rng = Self::from_seed(state.seed);
        rng.generator.set_stream(state.stream);
        rng.generator.set_word_pos(state.word_position);
        Ok(rng)
    }

    pub fn sample<T>(&mut self) -> T
    where
        rand::distr::StandardUniform: rand::distr::Distribution<T>,
    {
        self.generator.random()
    }

    pub fn range<T, R>(&mut self, range: R) -> T
    where
        T: rand::distr::uniform::SampleUniform,
        R: rand::distr::uniform::SampleRange<T>,
    {
        self.generator.random_range(range)
    }
}
