fn adder(a: felt252, b: felt252) -> felt252 {
    a + b
}

fn always_five(a: felt252, b: felt252) -> felt252{
    5
}

fn fib(a: felt252, b: felt252, n: felt252) -> felt252 {
    match n {
        0 => a,
        _ => fib(b, a + b, n - 1),
    }
}


#[cfg(test)]
mod tests {
    use super::adder;
    use super::always_five;
    use snforge_std::PrintTrait;

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

    #[test]
    #[fuzzer(runs: 10, seed: 100)]
    fn custom_fuzzer_config(b: felt252) {
        let result = adder(2, b);
        assert(result == 2 + b, '2 + b == 2 + b');
    }

    #[test]
    fn uint8_arg(a: u8) {
        if a <= 5_u8 {
            assert(2 == 2, '2 == 2');
        } else {
            let x = a - 5_u8;
            assert(x == a - 5_u8, 'x != a - 5');
        }
    }

    #[test]
    fn uint16_arg(a: u16) {
        if a <= 5_u16 {
            assert(2 == 2, '2 == 2');
        } else {
            let x = a - 5_u16;
            assert(x == a - 5_u16, 'x != a - 5');
        }
    }

    #[test]
    fn uint32_arg(a: u32) {
        if a <= 5_u32 {
            assert(2 == 2, '2 == 2');
        } else {
            let x = a - 5_u32;
            assert(x == a - 5_u32, 'x != a - 5');
        }
    }

    #[test]
    fn uint64_arg(a: u64) {
        if a <= 5_u64 {
            assert(2 == 2, '2 == 2');
        } else {
            let x = a - 5_u64;
            assert(x == a - 5_u64, 'x != a - 5');
        }
    }

    #[test]
    fn uint128_arg(a: u128) {
        if a <= 5_u128 {
            assert(2 == 2, '2 == 2');
        } else {
            let x = a - 5_u128;
            assert(x == a - 5_u128, 'x != a - 5');
        }
    }

    #[test]
    fn uint256_arg(a: u256) {
        if a <= 5_u256 {
            assert(2 == 2, '2 == 2');
        } else {
            let x = a - 5_u256;
            assert(x == a - 5_u256, 'x != a - 5');
        }
    }
}
