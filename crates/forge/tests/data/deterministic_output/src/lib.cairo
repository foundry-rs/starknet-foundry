fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, a + b, n - 1),
    }
}


#[cfg(test)]
mod test {
    use super::fib;

    #[test]
    fn second_test_pass_x() {
        fib(0, 1, 750000);
        assert(2 == 2, 'simple check');
    }

    #[test]
    fn first_test_pass_y() {
        fib(0, 1, 3);
        assert(2 == 2, 'simple check');
    }

    #[test]
    fn second_test_fail_y() {
        fib(0, 1, 750000);
        assert(1 == 2, 'simple check');
    }

    #[test]
    fn first_test_fail_x() {
        fib(0, 1, 3);
        assert(1 == 2, 'simple check');
    }
}
