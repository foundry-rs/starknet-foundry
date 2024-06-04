
#[cfg(test)]
mod xd {
    #[test]
    fn test_only() {
        assert(true, '');
    }

    fn non_test() {
        assert(true, '');
    }

    #[available_gas(1234567876543456)]
    #[test]
    fn with_args(a: felt252, b: u8) {
        assert(true, '');
    }

}