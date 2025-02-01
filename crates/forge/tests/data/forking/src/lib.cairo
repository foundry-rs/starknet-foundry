#[cfg(test)]
mod tests {
//     use starknet::ContractAddress;
//     use starknet::contract_address_const;


    // #[starknet::interface]
    // trait IHelloStarknet<TContractState> {
    //     fn increase_balance(ref self: TContractState, amount: felt252);
    //     fn get_balance(self: @TContractState) -> felt252;
    // }

    // #[test]
    // fn test_fork_simple() {
    //     let dispatcher = IHelloStarknetDispatcher {
    //         contract_address: contract_address_const::<
    //             0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9
    //         >()
    //     };

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 0, 'Balance should be 0');

    //     dispatcher.increase_balance(100);

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 100, 'Balance should be 100');
    // }

    // #[test]
    // fn test_fork_simple_number_hex() {
    //     let dispatcher = IHelloStarknetDispatcher {
    //         contract_address: contract_address_const::<
    //             0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9
    //         >()
    //     };

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 0, 'Balance should be 0');

    //     dispatcher.increase_balance(100);

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 100, 'Balance should be 100');
    // }

    // #[test]
    // fn test_fork_simple_hash_hex() {
    //     let dispatcher = IHelloStarknetDispatcher {
    //         contract_address: contract_address_const::<
    //             0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9
    //         >()
    //     };

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 0, 'Balance should be 0');

    //     dispatcher.increase_balance(100);

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 100, 'Balance should be 100');
    // }

    // #[test]
    // fn test_fork_simple_hash_number() {
    //     let dispatcher = IHelloStarknetDispatcher {
    //         contract_address: contract_address_const::<
    //             0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9
    //         >()
    //     };

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 0, 'Balance should be 0');

    //     dispatcher.increase_balance(100);

    //     let balance = dispatcher.get_balance();
    //     assert(balance == 100, 'Balance should be 100');
    // }

    #[test]
    fn print_block_number_when_latest() {
        assert(1 == 1, '');
    }
}
