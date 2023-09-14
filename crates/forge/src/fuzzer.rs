use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::{thread_rng, RngCore, SeedableRng};

pub struct Random {
    rng: Box<dyn RngCore>,
}

impl Random {
    pub fn new() -> Self {
        Random {
            rng: Box::new(thread_rng()),
        }
    }

    pub fn from_seed(seed: u64) -> Self {
        Random {
            rng: Box::new(StdRng::seed_from_u64(seed)),
        }
    }
}

impl Random {
    pub fn next_felt252(&mut self) -> Felt252 {
        let low = BigUint::zero();
        let high = Felt252::prime();

        let random_uint: BigUint = self.rng.gen_biguint_range(&low, &high);
        Felt252::from(random_uint)
    }
}
