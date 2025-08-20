#[starknet::interface]
trait ICheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatCallerAddressCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheckerMetaTxV0 of super::ICheckerMetaTxV0<ContractState> {
        fn __execute__(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }
    }
}

#[starknet::contract(account)]
mod TxInfoCheckerMetaTxV0 {
    use starknet::get_execution_info;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ITxInfoCheckerMetaTxV0 of super::ICheckerMetaTxV0<ContractState> {
        fn __execute__(ref self: ContractState) -> felt252 {
            let execution_info = get_execution_info().unbox();
            let tx_info = execution_info.tx_info.unbox();
            tx_info.version
        }
    }
}

#[starknet::interface]
trait ICheatBlockHashCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState, block_number: u64) -> felt252;
}

#[starknet::contract(account)]
mod CheatBlockHashCheckerMetaTxV0 {
    use starknet::SyscallResultTrait;
    use starknet::syscalls::get_block_hash_syscall;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockHashCheckerMetaTxV0 of super::ICheatBlockHashCheckerMetaTxV0<ContractState> {
        fn __execute__(ref self: ContractState, block_number: u64) -> felt252 {
            let block_hash = get_block_hash_syscall(block_number).unwrap_syscall();

            block_hash
        }
    }
}
