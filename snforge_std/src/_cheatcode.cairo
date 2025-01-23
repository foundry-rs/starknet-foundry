pub(crate) fn execute_cheatcode<const selector: felt252>(input: Span<felt252>) -> Span<felt252> {
    let result = starknet::testing::cheatcode::<selector>(input);
    let enum_variant = *result.at(0);
    let result_content = result.slice(1, result.len() - 1);

    if enum_variant == 1 { // Check if the result is an `Err`
        let mut arr = array![core::byte_array::BYTE_ARRAY_MAGIC];

        arr.append_span(result_content);

        panic(arr)
    } else {
        result_content
    }
}

pub(crate) fn execute_cheatcode_and_deserialize<const selector: felt252, T, +Serde<T>>(
    input: Span<felt252>
) -> T {
    let mut serialized_output = execute_cheatcode::<selector>(input);

    match Serde::deserialize(ref serialized_output) {
        Option::Some(output) => output,
        Option::None => panic!("snforge_std version mismatch: check the warning above")
    }
}

// Do not use this function directly.
// It is an internal part of the snforge architecture used by macros.
pub fn _is_config_run() -> bool {
    execute_cheatcode_and_deserialize::<'is_config_mode', bool>(array![].span())
}
