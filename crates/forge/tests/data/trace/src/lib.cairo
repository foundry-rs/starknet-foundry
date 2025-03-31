use starknet::ContractAddress;

#[derive(Drop, Serde, Clone)]
pub struct RecursiveCall {
    pub contract_address: ContractAddress,
    pub payload: Array<RecursiveCall>
}

#[starknet::interface]
pub trait RecursiveCaller<T> {
    fn execute_calls(self: @T, calls: Array<RecursiveCall>);
}

#[starknet::interface]
pub trait Failing<TContractState> {
    fn fail(self: @TContractState, data: Array<felt252>);
}

#[starknet::contract]
pub mod SimpleContract {
    use core::array::ArrayTrait;
    use super::{
        RecursiveCaller, RecursiveCallerDispatcher, RecursiveCallerDispatcherTrait, RecursiveCall,
        Failing,
    };


    #[storage]
    struct Storage {}


    #[abi(embed_v0)]
    impl RecursiveCallerImpl of RecursiveCaller<ContractState> {
        fn execute_calls(self: @ContractState, calls: Array<RecursiveCall>) {
            let mut i = 0;
            while i < calls.len() {
                let serviced_call = calls.at(i);
                RecursiveCallerDispatcher {
                    contract_address: serviced_call.contract_address.clone(),
                }
                    .execute_calls(serviced_call.payload.clone());
                i = i + 1;
            }
        }
    }

    #[abi(embed_v0)]
    impl FailingImpl of Failing<ContractState> {
        fn fail(self: @ContractState, data: Array<felt252>) {
            panic(data);
        }
    }
}
