fn adder(a: felt252, b: felt252) -> felt252 {
    a + b
}

fn always_five(a: felt252, b: felt252) -> felt252{
    5
}

#[cfg(test)]
mod tests {
    use super::adder;
    use super::always_five;
    use snforge_std::io::PrintTrait;

    #[test]
    fn adding() {
        let result = adder(2, 3);
        assert(result == 5, '2 + 3 == 5');
    }

    #[test]
    fn fuzzed_argument(b: felt252) {
        let result = adder(2, b);
        assert(result == 2 + b, '2 + b == 2 + b');
    }

    #[test]
    fn fuzzed_both_arguments(a: felt252, b: felt252) {
        let result = adder(a, b);
        assert(result == a + b, 'result == a + b');
    }

    #[test]
    fn passing() {
        let result = always_five(2, 3);
        assert(result == 5, 'result == 5');
    }

    #[test]
    fn failing_fuzz(a: felt252, b: felt252) {
        let result = always_five(a, b);
        assert(result == a + b, 'result == a + b');
    }
}
