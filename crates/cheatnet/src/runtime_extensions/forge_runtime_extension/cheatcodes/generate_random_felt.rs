use num_bigint::{BigUint, RandBigInt};
use starknet_types_core::felt::Felt;

#[must_use]
pub fn generate_random_felt() -> Felt {
    let mut rng = rand::thread_rng();

    let random_number: BigUint = rng.gen_biguint(251);
    Felt::from(random_number)
}
