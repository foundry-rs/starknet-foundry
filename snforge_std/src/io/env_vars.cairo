use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use option::OptionTrait;
use serde::Serde;

use starknet::testing::cheatcode;

fn read_env_var(name: felt252) -> felt252 {
    let outputs = cheatcode::<'read_env_var'>(array![name].span());
    *outputs[0]
}
