use fuzzing::adder;
use fuzzing::fib;

#[available_gas(l2_gas: 40000000)]
#[fuzzer(runs: 50, seed: 123)]
#[test]
fn with_available_gas(a: usize) {
    fib(0, 1, 1000);
    assert(a >= 0, 'unsigned must be >= 0');
}


#[fuzzer]
#[test]
#[should_panic(expected: 'panic message')]
fn with_should_panic(a: u64) {
    let b: i128 = a.into();
    assert(b < 0, 'panic message');
}

#[available_gas(l2_gas: 5)]
#[should_panic(expected: 'panic message')]
#[test]
#[fuzzer(runs: 300)]
fn with_both(a: felt252, b: u128) {
    let sum = adder(a, b.into());
    assert(sum + 1 == 0, 'panic message');
}

#[test]
#[fuzzer(seed: 5)]
#[ignore]
fn ignored(a: felt252) {
    assert(1 == 1, '');
}
