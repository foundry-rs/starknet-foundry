#[cfg(test)]
mod tests {
    use starknet::ContractAddress;
    use starknet::contract_address_const;
    use snforge_std::{BlockTag, BlockId};

    #[starknet::interface]
    trait IHelloStarknet<TContractState> {
        fn increase_balance(ref self: TContractState, amount: felt252);
        fn get_balance(self: @TContractState) -> felt252;
    }

    #[test]
    #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(313388))]
    fn test_fork_simple() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: contract_address_const::<
                3216637956526895219277698311134811322769343974163380838558193911733621219342
            >()
        };

        let balance = dispatcher.get_balance();
        assert(balance == 2, 'Balance should be 2');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 102, 'Balance should be 102');
    }

    #[test]
    #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Number(0x4c82c))]
    fn test_fork_simple_number_hex() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: contract_address_const::<
                3216637956526895219277698311134811322769343974163380838558193911733621219342
            >()
        };

        let balance = dispatcher.get_balance();
        assert(balance == 2, 'Balance should be 2');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 102, 'Balance should be 102');
    }

    #[test]
    #[fork(
        url: "http://188.34.188.184:9545/rpc/v0_6",
        block_id: BlockId::Hash(0x05a49d0e9704b2d5df7aed50551d96d138ad2a3525c9e3d3511fb42bf54f6b84)
    )]
    fn test_fork_simple_hash_hex() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: contract_address_const::<
                3216637956526895219277698311134811322769343974163380838558193911733621219342
            >()
        };

        let balance = dispatcher.get_balance();
        assert(balance == 2, 'Balance should be 2');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 102, 'Balance should be 102');
    }

    #[test]
    #[fork(
        url: "http://188.34.188.184:9545/rpc/v0_6",
        block_id: BlockId::Hash(
            2552411129059775354525588266083178415167802891503196038260593864007232744324
        )
    )]
    fn test_fork_simple_hash_number() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: contract_address_const::<
                3216637956526895219277698311134811322769343974163380838558193911733621219342
            >()
        };

        let balance = dispatcher.get_balance();
        assert(balance == 2, 'Balance should be 2');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 102, 'Balance should be 102');
    }

    #[test]
    #[fork(url: "http://188.34.188.184:9545/rpc/v0_6", block_id: BlockId::Tag(Latest))]
    fn print_block_number_when_latest() {
        assert(1 == 1, '');
    }
}
