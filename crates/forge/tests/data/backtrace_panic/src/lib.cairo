#[starknet::interface]
pub trait IOuterContract<TState> {
    fn outer(self: @TState, contract_address: starknet::ContractAddress);
}

#[starknet::contract]
pub mod OuterContract {
    use super::{IInnerContractDispatcher, IInnerContractDispatcherTrait};

    #[storage]
    pub struct Storage {}

    #[abi(embed_v0)]
    impl OuterContract of super::IOuterContract<ContractState> {
        fn outer(self: @ContractState, contract_address: starknet::ContractAddress) {
            let dispatcher = IInnerContractDispatcher { contract_address };
            dispatcher.inner();
        }
    }
}

#[starknet::interface]
pub trait IInnerContract<TState> {
    fn inner(self: @TState);
}

#[starknet::contract]
pub mod InnerContract {
    #[storage]
    pub struct Storage {}

    #[abi(embed_v0)]
    impl InnerContract of super::IInnerContract<ContractState> {
        fn inner(self: @ContractState) {
            inner_call()
        }
    }

    fn inner_call() {
        assert(1 != 1, 'Assert failed');
    }
}

#[cfg(test)]
mod Test {
    use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
    use snforge_std::{ContractClassTrait, declare};
    use super::{IOuterContractDispatcher, IOuterContractDispatcherTrait};

    #[test]
    fn test_contract_panics() {
        let contract_inner = declare("InnerContract").unwrap().contract_class();
        let (contract_address_inner, _) = contract_inner.deploy(@array![]).unwrap();

        let contract_outer = declare("OuterContract").unwrap().contract_class();
        let (contract_address_outer, _) = contract_outer.deploy(@array![]).unwrap();

        let dispatcher = IOuterContractDispatcher { contract_address: contract_address_outer };
        dispatcher.outer(contract_address_inner);
    }

    #[ignore]
    #[should_panic]
    #[test]
    fn test_contract_panics_with_should_panic() {
        let contract_inner = declare("InnerContract").unwrap().contract_class();
        let (contract_address_inner, _) = contract_inner.deploy(@array![]).unwrap();

        let contract_outer = declare("OuterContract").unwrap().contract_class();
        let (contract_address_outer, _) = contract_outer.deploy(@array![]).unwrap();

        let dispatcher = IOuterContractDispatcher { contract_address: contract_address_outer };
        dispatcher.outer(contract_address_inner);
    }

    #[test]
    #[fork(url: "{{ NODE_RPC_URL }}", block_number: 806134)]
    fn test_fork_contract_panics() {
        // NOTE: This is not exactly the same as InnerContract here, but will return the same error
        // Class hash needs to be different otherwise it would be found locally in the state
        let contract_address_inner =
            0x066eda239a01152a912129fe6b5bf309c9b21e3f583df4e5b7ee8ede1fad820a
            .try_into()
            .unwrap();

        let contract_outer = declare("OuterContract").unwrap().contract_class();
        let (contract_address_outer, _) = contract_outer.deploy(@array![]).unwrap();

        let dispatcher = IOuterContractDispatcher { contract_address: contract_address_outer };
        dispatcher.outer(contract_address_inner);
    }
}
