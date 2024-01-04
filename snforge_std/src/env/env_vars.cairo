use core::array::ArrayTrait;
use core::array::SpanTrait;
use core::clone::Clone;
use core::option::OptionTrait;
use core::serde::Serde;

use starknet::testing::cheatcode;

fn var(name: felt252) -> felt252 {
    let outputs = cheatcode::<'var'>(array![name].span());
    *outputs[0]
}
