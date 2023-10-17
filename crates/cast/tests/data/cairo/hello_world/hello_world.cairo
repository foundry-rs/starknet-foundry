use debug::PrintTrait;
use traits::Into;

use starknet::testing::cheatcode;

fn print_felt252(felt_: felt252) {
    cheatcode::<'print_felt252'>(array![felt_].span());
}

fn main() {
    'Hello, World!'.print();
    print_felt252('Hello, cheatcode!');
}
