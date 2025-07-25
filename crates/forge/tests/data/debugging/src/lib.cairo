use starknet::ContractAddress;

#[derive(Drop, Serde, Clone)]
struct RecursiveCall {
    contract_address: ContractAddress,
    payload: Array<RecursiveCall>,
}

#[starknet::interface]
trait RecursiveCaller<T> {
    fn execute_calls(self: @T, calls: Array<RecursiveCall>) -> Array<RecursiveCall>;
}

#[starknet::interface]
trait Failing<TContractState> {
    fn fail(self: @TContractState, data: Array<felt252>);
}

#[starknet::contract]
mod SimpleContract {
    use core::array::ArrayTrait;
    use core::traits::Into;
    use starknet::{ContractAddress, get_contract_address};
    use super::{
        Failing, RecursiveCall, RecursiveCaller, RecursiveCallerDispatcher,
        RecursiveCallerDispatcherTrait,
    };


    #[storage]
    struct Storage {}


    #[abi(embed_v0)]
    impl RecursiveCallerImpl of RecursiveCaller<ContractState> {
        fn execute_calls(
            self: @ContractState, calls: Array<RecursiveCall>,
        ) -> Array<RecursiveCall> {
            let mut i = 0;
            #[cairofmt::skip]
            while i < calls.len() {
                let serviced_call = calls.at(i);
                RecursiveCallerDispatcher {
                    contract_address: serviced_call.contract_address.clone(),
                }
                    .execute_calls(serviced_call.payload.clone());
                i = i + 1;
            };

            calls
        }
    }

    #[abi(embed_v0)]
    impl FailingImpl of Failing<ContractState> {
        fn fail(self: @ContractState, data: Array<felt252>) {
            panic(data);
        }
    }
}
