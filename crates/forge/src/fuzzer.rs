use cairo_felt::Felt252;
use num_bigint::{BigUint, RandBigInt};
use num_traits::Zero;
use rand::rngs::StdRng;
use rand::{thread_rng, RngCore, SeedableRng};

pub struct Random {
    rng: StdRng,
    seed: u64,
}

impl Random {
    pub fn new() -> Self {
        let mut seed_rng = thread_rng();
        let seed = seed_rng.next_u64();

        let rng = StdRng::seed_from_u64(seed);

        Random { rng, seed }
    }

    pub fn from_seed(seed: u64) -> Self {
        Random {
            rng: StdRng::seed_from_u64(seed),
            seed,
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

    pub fn seed(&self) -> u64 {
        self.seed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_is_set() {
        let fuzzer = Random::from_seed(12345);
        assert_eq!(fuzzer.seed, 12345);
        assert_eq!(fuzzer.seed, fuzzer.seed());
    }

    #[test]
    fn using_seed_consistent_result() {
        let mut fuzzer = Random::new();
        let values: Vec<Felt252> = (1..100).map(|_| fuzzer.next_felt252()).collect();

        let mut fuzzer_from_seed = Random::from_seed(fuzzer.seed);
        let values_from_seed: Vec<Felt252> =
            (1..100).map(|_| fuzzer_from_seed.next_felt252()).collect();

        assert_eq!(values, values_from_seed);
    }
}
