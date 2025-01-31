use exit_first::fib;

#[test]
fn hard_test() {
    fib(0, 1, 99999);
    assert(2 == 2, 'simple check');
}

#[test]
fn simple_test() {
    fib(0, 1, 3);
    assert(2 == 2, 'simple check');
}
