use array::ArrayTrait;
use clone::Clone;
use snforge_std::PrintTrait;
use core::traits::Into;

use starknet::testing::cheatcode;

#[derive(Drop, Copy)]
enum Tag {
    Last: (),
    Pending: (),
}

#[derive(Drop, Copy)]
enum BlockId {
    Tag: Tag,
    Hash: felt252,
    Number: felt252,
}

#[derive(Drop, Clone, Copy)]
struct ForkConfig {
   url: felt252,
   block: BlockId,
}

trait ForkTrait {
    fn set_up(self: ForkConfig) -> ForkConfig;
}

impl ForkImpl of ForkTrait {
    fn set_up(self: ForkConfig) -> ForkConfig {
        let mut inputs: Array::<felt252> = ArrayTrait::new();

        inputs.append(self.url.clone().into());

        match self.block {
            BlockId::Tag(tag) => {
                inputs.append(0.into());
                match tag {
                    Tag::Last(()) => {
                        inputs.append(0.into());
                    },
                    Tag::Pending(()) => {
                        inputs.append(1.into());
                    },
                }
            },
            BlockId::Hash(hash) => {
                inputs.append(1.into());
                let mut h = hash.clone();
                inputs.append(h.into());
            },
            BlockId::Number(number) => {
                inputs.append(2.into());
                let mut n = number.clone();
                inputs.append(n.into());
            },
        };

        cheatcode::<'setup_fork'>(inputs.span());

        self
    }
}
