#[starknet::interface]
trait IMetaTxV0Test<TContractState> {
    fn execute_meta_tx_get_caller(
        ref self: TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_block_number(
        ref self: TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_block_timestamp(
        ref self: TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_sequencer_address(
        ref self: TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_block_hash(
        ref self: TContractState,
        target: starknet::ContractAddress,
        block_number: u64,
        signature: Span<felt252>,
    ) -> felt252;
    fn get_caller_address(ref self: TContractState) -> felt252;
    fn get_block_number(ref self: TContractState) -> u64;
    fn get_block_timestamp(ref self: TContractState) -> u64;
    fn get_sequencer_address(ref self: TContractState) -> starknet::ContractAddress;
    fn get_block_hash(ref self: TContractState, block_number: u64) -> felt252;
}

#[starknet::contract]
mod MetaTxV0Test {
    use starknet::syscalls::meta_tx_v0_syscall;

    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState) {}

    #[abi(embed_v0)]
    impl IMetaTxV0Test of super::IMetaTxV0Test<ContractState> {
        fn execute_meta_tx_get_caller(
            ref self: ContractState,
            target: starknet::ContractAddress,
            signature: Span<felt252>,
        ) -> felt252 {
            let selector = selector!("get_caller_address");
            let calldata: Array<felt252> = array![];
            
            let result = meta_tx_v0_syscall(target, selector, calldata.span(), signature)
                .unwrap();
            
            *result.at(0)
        }

        fn execute_meta_tx_get_block_number(
            ref self: ContractState,
            target: starknet::ContractAddress,
            signature: Span<felt252>,
        ) -> felt252 {
            let selector = selector!("get_block_number");
            let calldata: Array<felt252> = array![];
            
            let result = meta_tx_v0_syscall(target, selector, calldata.span(), signature)
                .unwrap();
            
            *result.at(0)
        }

        fn execute_meta_tx_get_block_timestamp(
            ref self: ContractState,
            target: starknet::ContractAddress,
            signature: Span<felt252>,
        ) -> felt252 {
            let selector = selector!("get_block_timestamp");
            let calldata: Array<felt252> = array![];
            
            let result = meta_tx_v0_syscall(target, selector, calldata.span(), signature)
                .unwrap();
            
            *result.at(0)
        }

        fn execute_meta_tx_get_sequencer_address(
            ref self: ContractState,
            target: starknet::ContractAddress,
            signature: Span<felt252>,
        ) -> felt252 {
            let selector = selector!("get_sequencer_address");
            let calldata: Array<felt252> = array![];
            
            let result = meta_tx_v0_syscall(target, selector, calldata.span(), signature)
                .unwrap();
            
            *result.at(0)
        }

        fn execute_meta_tx_get_block_hash(
            ref self: ContractState,
            target: starknet::ContractAddress,
            block_number: u64,
            signature: Span<felt252>,
        ) -> felt252 {
            let selector = selector!("get_block_hash");
            let calldata: Array<felt252> = array![block_number.into()];
            
            let result = meta_tx_v0_syscall(target, selector, calldata.span(), signature)
                .unwrap();
            
            *result.at(0)
        }

        fn get_caller_address(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }

        fn get_block_number(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_number
        }

        fn get_block_timestamp(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_timestamp
        }

        fn get_sequencer_address(ref self: ContractState) -> starknet::ContractAddress {
            starknet::get_block_info().unbox().sequencer_address
        }

        fn get_block_hash(ref self: ContractState, block_number: u64) -> felt252 {
            starknet::syscalls::get_block_hash_syscall(block_number).unwrap()
        }
    }
}
