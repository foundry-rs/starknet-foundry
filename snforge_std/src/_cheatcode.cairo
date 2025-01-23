pub(crate) fn execute_cheatcode<const selector: felt252>(input: Span<felt252>) -> Span<felt252> {
    let output = starknet::testing::cheatcode::<selector>(input);
    let first = *output.at(0);
    let output = output.slice(1, output.len() - 1);

    if first == 1 {
        let mut arr = array![core::byte_array::BYTE_ARRAY_MAGIC];

        arr.append_span(output);

        panic(arr)
    } else {
        output
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
