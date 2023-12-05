use fuzzing::adder;
use fuzzing::fib;


#[test]
fn exit_first_fails_test(b: felt252) {
    adder(0, 1);
    assert(1 == 2, '2 + b == 2 + b');
}

#[test]
fn exit_first_hard_test(b: felt252) {
    fib(0, 1, 30344);
    assert(2 == 2, 'simple check');
}
