#[starknet::interface]
pub trait IErrorHandler<TContractState> {
    fn catch_panic_and_handle(self: @TContractState);
    fn catch_panic_and_fail(self: @TContractState);
    fn call_unrecoverable(self: @TContractState);
}

#[feature("safe_dispatcher")]
#[starknet::contract]
pub mod ErrorHandler {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use crate::failable::{IFailableContractSafeDispatcher, IFailableContractSafeDispatcherTrait};
    use core::panic_with_felt252;

    #[storage]
    pub struct Storage {
        failable_address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, failable_address: ContractAddress) {
        self.failable_address.write(failable_address);
    }


    #[abi(embed_v0)]
    impl ErrorHandler of super::IErrorHandler<ContractState> {
        fn catch_panic_and_handle(self: @ContractState) {
            let dispatcher = get_safe_dispatcher(self);

            match dispatcher.recoverable_panic() {
                Result::Ok(_) => panic_with_felt252('Expected panic'),
                Result::Err(panic_data) => {
                    assert(*panic_data.at(0) == 'Errare humanum est', 'Incorrect error');
                    assert(*panic_data.at(1) == 'ENTRYPOINT_FAILED', 'Missing generic error');
                    assert(panic_data.len() == 2, 'Incorrect error length');
                },
            }
        }

        fn catch_panic_and_fail(self: @ContractState) {
            let dispatcher = get_safe_dispatcher(self);

            match dispatcher.recoverable_panic() {
                Result::Ok(_) => panic_with_felt252('Expected panic'),
                Result::Err(panic_data) => {
                    assert(*panic_data.at(0) == 'Errare humanum est', 'Incorrect error');
                    assert(*panic_data.at(1) == 'ENTRYPOINT_FAILED', 'Missing generic error');
                    assert(panic_data.len() == 2, 'Incorrect error length');
                },
            }

            panic_with_felt252('Different panic');
        }

        fn call_unrecoverable(self: @ContractState) {
            let dispatcher = get_safe_dispatcher(self);

            match dispatcher.unrecoverable_error() {
                // Unreachable
                Result::Ok(_) => {},
                Result::Err(_) => {},
            }
        }
    }

    fn get_safe_dispatcher(self: @ContractState) -> IFailableContractSafeDispatcher {
        IFailableContractSafeDispatcher { contract_address: self.failable_address.read() }
    }
}
