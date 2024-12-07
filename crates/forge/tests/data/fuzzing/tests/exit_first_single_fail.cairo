
use fuzzing::fib;




#[test]
fn exit_first_hard_test(b: felt252) {
    fib(0, 1, 30344);
    assert(2 == 2, 'simple check');
}
