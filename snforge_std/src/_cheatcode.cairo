use core::array::ArrayTrait;
use core::traits::Into;
use core::array::SpanTrait;

pub fn handle_cheatcode(input: Span<felt252>) -> Span<felt252> {
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
