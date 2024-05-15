use starknet::ClassHash;

#[starknet::interface]
trait ICheatTxInfoChecker<TContractState> {
    fn get_transaction_hash(self: @TContractState) -> felt252;
}

#[starknet::interface]
trait ICheatTxInfoCheckerLibCall<TContractState> {
    fn get_tx_hash_with_lib_call(self: @TContractState, class_hash: ClassHash) -> felt252;
    fn get_tx_info(self: @TContractState) -> starknet::info::v2::TxInfo;
}

#[starknet::contract]
mod CheatTxInfoCheckerLibCall {
    use super::{ICheatTxInfoCheckerDispatcherTrait, ICheatTxInfoCheckerLibraryDispatcher};
    use starknet::{
        ClassHash, SyscallResultTrait, SyscallResult, syscalls::get_execution_info_v2_syscall
    };

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatTxInfoCheckerLibCall of super::ICheatTxInfoCheckerLibCall<ContractState> {
        fn get_tx_hash_with_lib_call(self: @ContractState, class_hash: ClassHash) -> felt252 {
            let tx_info_checker = ICheatTxInfoCheckerLibraryDispatcher { class_hash };
            tx_info_checker.get_transaction_hash()
        }

        fn get_tx_info(self: @ContractState) -> starknet::info::v2::TxInfo {
            let execution_info = get_execution_info_v2_syscall().unwrap_syscall().unbox();
            execution_info.tx_info.unbox()
        }
    }
}
