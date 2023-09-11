use array::ArrayTrait;
use clone::Clone;
use snforge_std::PrintTrait;

use starknet::testing::cheatcode;

#[derive(Drop, Clone)]
struct ForkConfig {
   env_with_url: felt252,
   block: felt252,
}

trait ForkTrait {
    fn set_up(self: @ForkConfig) -> felt252;
}

impl ForkImpl of ForkTrait {
    fn set_up(self: @ForkConfig) -> felt252 {
        'dupa3'.print();
        let mut url = self.env_with_url.clone();
        let mut block = self.block.clone();
        let mut inputs = array![
            url,
            block
        ];

        'dupa'.print();

        cheatcode::<'setup_fork'>(inputs.span());

        '232'
    }
}
