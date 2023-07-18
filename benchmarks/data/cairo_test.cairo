#[cfg(test)]
mod hello_starknet_test {
    use starknet::testing::{set_caller_address, set_contract_address, set_block_timestamp};

    use project::HelloStarknet;
    use project::{IHelloStarknetDispatcher, IHelloStarknetDispatcherTrait};

    use starknet::{
        contract_address_const, get_block_info, ContractAddress, Felt252TryIntoContractAddress,
        TryInto, Into, OptionTrait, class_hash::Felt252TryIntoClassHash
    };
    use starknet::storage_read_syscall;

    use starknet::syscalls::deploy_syscall;
    use array::{ArrayTrait, SpanTrait, ArrayTCloneImpl};
    use result::ResultTrait;


    fn deploy_hello_starknet() -> ContractAddress {
        let account: ContractAddress = contract_address_const::<1>();
        // Set account as default caller
        set_caller_address(account);

        let mut calldata = ArrayTrait::new();

        let result = deploy_syscall(
            HelloStarknet::TEST_CLASS_HASH.try_into().unwrap(), 0, calldata.span(), false
        );

        let (address, _) = match result {
            Result::Ok(x) => x,
            Result::Err(x) => panic_with_felt252(*x.at(0_usize))
        };

        address
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_1() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_2() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_3() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_4() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_5() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_6() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_7() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_8() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_9() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_10() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_11() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_12() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_13() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_14() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

    #[test]
    #[available_gas(1000000)]
    fn test_increase_balance_15() {
        let contract_address = deploy_hello_starknet();

        let dispatcher = IHelloStarknetDispatcher { contract_address };

        let balance_before = dispatcher.get_balance();
        assert(balance_before == 0, 'Invalid balance');

        dispatcher.increase_balance(42);

        let balance_after = dispatcher.get_balance();
        assert(balance_after == 42, 'Invalid balance');
    }

}