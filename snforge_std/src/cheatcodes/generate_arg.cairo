use super::super::_cheatcode::execute_cheatcode_and_deserialize;

// Generates a random number that is used for creating data for fuzz tests
pub fn generate_arg<T, +Serde<T>, +Drop<T>, +Into<T, felt252>>(min_value: T, max_value: T) -> T {
    execute_cheatcode_and_deserialize::<
        'generate_arg',
    >(array![min_value.into(), max_value.into()].span())
}
