use exit_first::fib;

#[test]
fn hard_test() {
    fib(0, 1, 30344);
    assert(2 == 2, 'simple check');
}

