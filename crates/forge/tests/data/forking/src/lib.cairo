#[cfg(test)]
mod tests {
    use starknet::contract_address_const;
    const CONTRACT_ADDRESS: felt252 =
        0x202de98471a4fae6bcbabb96cab00437d381abc58b02509043778074d6781e9;

    #[starknet::interface]
    trait IHelloStarknet<TContractState> {
        fn increase_balance(ref self: TContractState, amount: felt252);
        fn get_balance(self: @TContractState) -> felt252;
    }

    #[test]
    #[fork(url: "{{ NODE_RPC_URL }}", block_number: 54060)]
    fn test_fork_simple() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: CONTRACT_ADDRESS.try_into().unwrap(),
        };

        let balance = dispatcher.get_balance();
        assert(balance == 0, 'Balance should be 0');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 100, 'Balance should be 100');
    }

    #[test]
    #[fork(url: "{{ NODE_RPC_URL }}", block_number: 0xd32c)]
    fn test_fork_simple_number_hex() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: CONTRACT_ADDRESS.try_into().unwrap(),
        };

        let balance = dispatcher.get_balance();
        assert(balance == 0, 'Balance should be 0');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 100, 'Balance should be 100');
    }

    #[test]
    #[fork(
        url: "{{ NODE_RPC_URL }}",
        block_hash: 0x06ae121e46f5375f93b00475fb130348ae38148e121f84b0865e17542e9485de,
    )]
    fn test_fork_simple_hash_hex() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: CONTRACT_ADDRESS.try_into().unwrap(),
        };

        let balance = dispatcher.get_balance();
        assert(balance == 0, 'Balance should be 0');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 100, 'Balance should be 100');
    }

    #[test]
    #[fork(
        url: "{{ NODE_RPC_URL }}",
        block_hash: 3021433528476416000728121069095289682281028310523383289416465162415092565470,
    )]
    fn test_fork_simple_hash_number() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: CONTRACT_ADDRESS.try_into().unwrap(),
        };

        let balance = dispatcher.get_balance();
        assert(balance == 0, 'Balance should be 0');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 100, 'Balance should be 100');
    }

    #[test]
    #[fork(url: "{{ NODE_RPC_URL }}", block_tag: latest)]
    fn print_block_number_when_latest() {
        assert(1 == 1, '');
    }
}
