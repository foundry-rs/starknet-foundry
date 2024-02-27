use starknet::testing::cheatcode;
use super::super::byte_array::byte_array_as_felt_array;

fn var(name: ByteArray) -> felt252 {
    let outputs = cheatcode::<'var'>(byte_array_as_felt_array(@name).span());
    *outputs[0]
}
