use starknet::ContractAddress;

#[derive(Drop, Serde, Clone)]
struct RecursiveCall {
    contract_address: ContractAddress,
    payload: Array<RecursiveCall>,
}

#[starknet::interface]
trait RecursiveCaller<T> {
    fn execute_calls(ref self: T, calls: Array<RecursiveCall>) -> Array<RecursiveCall>;
}

#[starknet::interface]
trait Failing<TContractState> {
    fn fail(self: @TContractState, data: Array<felt252>);
}

#[starknet::contract]
mod SimpleContract {
    use core::array::ArrayTrait;
    use core::traits::Into;
    use starknet::ContractAddress;
    use super::{
        Failing, RecursiveCall, RecursiveCaller, RecursiveCallerDispatcher,
        RecursiveCallerDispatcherTrait,
    };


    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        CallsExecuted: CallsExecuted,
    }

    #[derive(Drop, starknet::Event)]
    struct CallsExecuted {
        calls_len: felt252,
    }

    #[abi(embed_v0)]
    impl RecursiveCallerImpl of RecursiveCaller<ContractState> {
        fn execute_calls(
            ref self: ContractState, calls: Array<RecursiveCall>,
        ) -> Array<RecursiveCall> {
            self.emit(Event::CallsExecuted(CallsExecuted { calls_len: calls.len().into() }));

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
