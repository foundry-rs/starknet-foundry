use array::ArrayTrait;
use clone::Clone;

#[derive(Drop, Clone)]
struct ForkConfig {
   env_with_url: felt252,
   block: felt252,
}

// trait ForkTrait {
//     fn set_up(self: @ForkConfig) -> @ForkConfig;
// }

// impl ForkImpl of ForkTrait {
//     fn set_up(self: @ForkConfig) -> @ForkConfig {
//         let mut inputs: Array::<felt252> = array![
//             self.env_with_url,
//             self.block,
//         ];

//         cheatcode::<'setup_fork'>(inputs.span());
//     }
// }

