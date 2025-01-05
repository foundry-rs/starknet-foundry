use core::option::OptionTrait;
use core::array::ArrayTrait;
use core::traits::Into;
use core::array::SpanTrait;

pub(crate) fn handle_cheatcode(input: Span<felt252>) -> Span<felt252> {
    let first = *input.at(0);
    let input = input.slice(1, input.len() - 1);

    if first == 1 {
        let mut arr = array![core::byte_array::BYTE_ARRAY_MAGIC];

        arr.append_span(input);

        panic(arr)
    } else {
        input
    }
}

// Do not use this function directly.
// It is an internal part of the snforge architecture used by macros.
pub fn _is_config_run() -> bool {
    let mut res = handle_cheatcode(
        starknet::testing::cheatcode::<'is_config_mode'>(array![].span())
    );

    Serde::deserialize(ref res).unwrap_or(false)
}
