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
    use starknet::{SyscallResultTrait, ContractAddress};
    use starknet::syscalls::call_contract_syscall;

    #[storage]
    pub struct Storage {}

    #[abi(embed_v0)]
    impl InnerContract of super::IInnerContract<ContractState> {
        fn inner(self: @ContractState) {
            inner_call()
        }
    }

    fn inner_call() {
        let address: ContractAddress = 0x123.try_into().unwrap();
        let selector = selector!("dummy");
        let calldata = array![].span();

        // This fails immediately due to nonexistent address
        call_contract_syscall(address, selector, calldata).unwrap_syscall();
    }
}

#[cfg(test)]
mod Test {
    use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
    use snforge_std::{ContractClassTrait, declare};
    use super::{IOuterContractDispatcher, IOuterContractDispatcherTrait};

    #[test]
    fn test_unwrapped_call_contract_syscall() {
        let contract_inner = declare("InnerContract").unwrap().contract_class();
        let (contract_address_inner, _) = contract_inner.deploy(@array![]).unwrap();

        let contract_outer = declare("OuterContract").unwrap().contract_class();
        let (contract_address_outer, _) = contract_outer.deploy(@array![]).unwrap();

        let dispatcher = IOuterContractDispatcher { contract_address: contract_address_outer };
        dispatcher.outer(contract_address_inner);
    }

    #[test]
    #[fork(url: "{{ NODE_RPC_URL }}", block_number: 806134)]
    fn test_fork_unwrapped_call_contract_syscall() {
        // NOTE: This is not exactly the same as InnerContract here, but will return the same error
        // Class hash needs to be different otherwise it would be found locally in the state
        let contract_address_inner =
            0x01506c04bdb56f2cc9ea1f67fcb086c99df7cec2598ce90e56f1d36fffda1cf4
            .try_into()
            .unwrap();

        let contract_outer = declare("OuterContract").unwrap().contract_class();
        let (contract_address_outer, _) = contract_outer.deploy(@array![]).unwrap();

        let dispatcher = IOuterContractDispatcher { contract_address: contract_address_outer };
        dispatcher.outer(contract_address_inner);
    }
}
