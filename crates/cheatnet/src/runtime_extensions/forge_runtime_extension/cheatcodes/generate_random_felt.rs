use cairo_vm::Felt252;
use num_bigint::{BigUint, RandBigInt};

pub fn generate_random_felt() -> Felt252 {
    let mut rng = rand::thread_rng();
    // Generates a random 252-bit number
    let random_number: BigUint = rng.gen_biguint(252);
    println!("Random number: {}", random_number);
    Felt252::from(random_number)
}
