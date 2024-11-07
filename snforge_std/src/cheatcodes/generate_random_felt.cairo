use starknet::testing::cheatcode;
use core::serde::Serde;
use super::super::_cheatcode::handle_cheatcode;


/// Generates a random felt value
///
/// Returns a random felt within the range of 0 and 2^252 - 1
fn generate_random_felt() -> felt252 {
    let mut random_felt = handle_cheatcode(cheatcode::<'generate_random_felt'>(array![].span()));

    Serde::deserialize(ref random_felt).unwrap()
}
