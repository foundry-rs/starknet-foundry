use array::ArrayTrait;
use array::SpanTrait;
use clone::Clone;
use option::OptionTrait;
use serde::Serde;

use starknet::testing::cheatcode;

fn var(name: felt252) -> felt252 {
    let outputs = cheatcode::<'var'>(array![name].span());
    *outputs[0]
}
