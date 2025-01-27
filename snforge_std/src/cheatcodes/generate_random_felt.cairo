use super::super::_cheatcode::execute_cheatcode_and_deserialize;


/// Generates a random felt value
///
/// Returns a random felt within the range of 0 and 2^252 - 1
pub fn generate_random_felt() -> felt252 {
    execute_cheatcode_and_deserialize::<'generate_random_felt'>(array![].span())
}
