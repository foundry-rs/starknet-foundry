use array::ArrayTrait;
use clone::Clone;
use snforge_std::PrintTrait;
use core::traits::Into;
use serde::Serde;

use starknet::testing::cheatcode;

#[derive(Drop, Copy, Serde)]
enum BlockTag {
    Latest,
    Pending,
}

#[derive(Drop, Copy, Serde)]
enum BlockId {
    Tag: BlockTag,
    Hash: felt252,
    Number: felt252,
}

#[derive(Drop, Clone, Serde)]
struct ForkConfig {
   url: Array::<felt252>,
   block: BlockId,
}

trait ForkTrait {
    fn set_up(self: ForkConfig);
}

impl ForkImpl of ForkTrait {
    fn set_up(self: ForkConfig) {
        let mut inputs = array![];
        self.serialize(ref inputs);

        cheatcode::<'setup_fork'>(inputs.span());
    }
}
