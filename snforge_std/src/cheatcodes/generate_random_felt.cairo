use super::super::_cheatcode::typed_checked_cheatcode;


/// Generates a random felt value
///
/// Returns a random felt within the range of 0 and 2^252 - 1
pub fn generate_random_felt() -> felt252 {
    typed_checked_cheatcode::<'generate_random_felt', felt252>(array![].span())
}
