#[cfg(test)]
mod tests {
    use starknet::ContractAddress;
    use starknet::contract_address_const;
    use snforge_std::{ BlockTag, BlockId };

    #[starknet::interface]
    trait IHelloStarknet<TContractState> {
        fn increase_balance(ref self: TContractState, amount: felt252);
        fn get_balance(self: @TContractState) -> felt252;
    }

    #[test]
    #[fork(url: "http://188.34.188.184:9545/rpc/v0.4", block_id: BlockId::Number(313388))]
    fn test_fork_simple() {
        let dispatcher = IHelloStarknetDispatcher {
            contract_address: contract_address_const::<3216637956526895219277698311134811322769343974163380838558193911733621219342>()
        };

        let balance = dispatcher.get_balance();
        assert(balance == 2, 'Balance should be 2');

        dispatcher.increase_balance(100);

        let balance = dispatcher.get_balance();
        assert(balance == 102, 'Balance should be 102');
    }
}
