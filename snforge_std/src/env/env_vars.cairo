use starknet::testing::cheatcode;

fn var(name: felt252) -> felt252 {
    let outputs = cheatcode::<'var'>(array![name].span());
    *outputs[0]
}
