use cairo_vm::Felt252;
use num_bigint::{BigUint, RandBigInt};

pub fn generate_random_felt() -> Felt252 {
    let mut rng = rand::thread_rng();

    let random_number: BigUint = rng.gen_biguint(251);
    Felt252::from(random_number)
}
