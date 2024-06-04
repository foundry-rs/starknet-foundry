
#[cfg(test)]
mod xd {
    #[test]
    fn test_only() {
        if snforge_std::_cheatcode::_is_config_run() {

            let mut data = array![];

            snforge_std::_config_types::ForkConfig::Named("test")
                .serialize(ref data);

            starknet::testing::cheatcode::<'set_config_fork'>(data.span());

            return;
        }
        assert(true, '');
    }

    fn non_test() {
        assert(true, '');
    }

    // #[available_gas(1234567876543456)]
    // #[test]
    // fn with_args(a: felt252, b: u8) {
    //     assert(true, '');
    // }

}