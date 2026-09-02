use bevy::prelude::*;
use rand_chacha::rand_core::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

#[derive(Resource)]
pub struct DeterministicRng {
    pub rng: ChaCha8Rng,
}

impl Default for DeterministicRng {
    fn default() -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(0x1337_D00D),
        }
    }
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.rng.next_u32() >> 8) as f32 / (1 << 24) as f32
    }
}
