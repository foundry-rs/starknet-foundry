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
            let selector = selector!("__execute__");
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
            let selector = selector!("__execute__");
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
            let selector = selector!("__execute__");
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
            let selector = selector!("__execute__");
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
            let selector = selector!("__execute__");
            let calldata: Array<felt252> = array![block_number.into()];
            
            let result = meta_tx_v0_syscall(target, selector, calldata.span(), signature)
                .unwrap();
            
            *result.at(0)
        }
    }
}
