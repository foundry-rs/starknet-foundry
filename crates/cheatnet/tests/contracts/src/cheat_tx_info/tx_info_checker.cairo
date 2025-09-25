use starknet::ContractAddress;

#[starknet::interface]
trait ICheatTxInfoChecker<TContractState> {
    fn get_account_contract_address(ref self: TContractState) -> ContractAddress;
    fn get_transaction_hash(self: @TContractState) -> felt252;
    fn get_tx_hash_and_emit_event(ref self: TContractState) -> felt252;
    fn get_tx_info(self: @TContractState) -> starknet::TxInfo;
}

#[starknet::contract]
mod CheatTxInfoChecker {
    use starknet::syscalls::get_execution_info_v2_syscall;
    use starknet::{ContractAddress, SyscallResultTrait, get_tx_info};

    #[storage]
    struct Storage {}

    #[event]
    #[derive(Drop, starknet::Event)]
    enum Event {
        TxHashEmitted: TxHashEmitted,
    }

    #[derive(Drop, starknet::Event)]
    struct TxHashEmitted {
        tx_hash: felt252,
    }

    #[abi(embed_v0)]
    impl ICheatTxInfoChecker of super::ICheatTxInfoChecker<ContractState> {
        fn get_transaction_hash(self: @ContractState) -> felt252 {
            starknet::get_tx_info().unbox().transaction_hash
        }

        fn get_tx_hash_and_emit_event(ref self: ContractState) -> felt252 {
            let tx_hash = starknet::get_tx_info().unbox().transaction_hash;
            self.emit(Event::TxHashEmitted(TxHashEmitted { tx_hash }));
            tx_hash
        }

        fn get_tx_info(self: @ContractState) -> starknet::TxInfo {
            let execution_info = get_execution_info_v2_syscall().unwrap_syscall().unbox();
            execution_info.tx_info.unbox()
        }

        fn get_account_contract_address(ref self: ContractState) -> ContractAddress {
            let tx_info = get_tx_info();
            tx_info.account_contract_address
        }
    }
}
