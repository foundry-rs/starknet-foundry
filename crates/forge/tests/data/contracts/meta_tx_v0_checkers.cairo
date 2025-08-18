#[starknet::interface]
trait ICheatCallerAddressCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatCallerAddressCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatCallerAddressCheckerMetaTxV0 of super::ICheatCallerAddressCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}

// CheatBlockNumberCheckerMetaTxV0
#[starknet::interface]
trait ICheatBlockNumberCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatBlockNumberCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockNumberCheckerMetaTxV0 of super::ICheatBlockNumberCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            let block_number = starknet::get_block_number();
            block_number.into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}

// CheatBlockTimestampCheckerMetaTxV0
#[starknet::interface]
trait ICheatBlockTimestampCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatBlockTimestampCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockTimestampCheckerMetaTxV0 of super::ICheatBlockTimestampCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            let block_timestamp = starknet::get_block_timestamp();
            block_timestamp.into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}

// CheatSequencerAddressCheckerMetaTxV0
#[starknet::interface]
trait ICheatSequencerAddressCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
}

#[starknet::contract(account)]
mod CheatSequencerAddressCheckerMetaTxV0 {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatSequencerAddressCheckerMetaTxV0 of super::ICheatSequencerAddressCheckerMetaTxV0<
        ContractState,
    > {
        fn __execute__(ref self: ContractState) -> felt252 {
            let sequencer_address = starknet::get_block_info().unbox().sequencer_address;
            sequencer_address.into()
        }

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}

// CheatBlockHashCheckerMetaTxV0
#[starknet::interface]
trait ICheatBlockHashCheckerMetaTxV0<TContractState> {
    fn __execute__(ref self: TContractState, block_number: u64) -> felt252;
    fn __validate__(ref self: TContractState) -> felt252;
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

        fn __validate__(ref self: ContractState) -> felt252 {
            starknet::VALIDATED
        }
    }
}
