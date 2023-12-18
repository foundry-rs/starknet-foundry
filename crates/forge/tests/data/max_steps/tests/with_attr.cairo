use max_steps::fib;

#[test]
#[max_steps(120)]
fn hard_test() {
    fib(0, 1, 30344);
    assert(2 == 2, 'simple check');
}

#[test]
#[max_steps(120)]
fn simple_test() {
    fib(0, 1, 3);
    assert(2 == 2, 'simple check');
}
