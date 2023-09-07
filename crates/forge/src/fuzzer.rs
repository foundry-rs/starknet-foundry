use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::{thread_rng, RngCore, SeedableRng};

pub trait Fuzzer {
    fn next_argument(&mut self, type_name: &str) -> Vec<Felt252>;
}

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

impl Fuzzer for Random {
    fn next_argument(&mut self, type_name: &str) -> Vec<Felt252> {
        assert_eq!(
            type_name, "felt252",
            "Types other than `felt252` are not supported by RandomFuzzer"
        );

        let low = BigUint::zero();
        let high = Felt252::prime();

        let random_uint: BigUint = self.rng.gen_biguint_range(&low, &high);
        vec![Felt252::from(random_uint)]
    }
}
