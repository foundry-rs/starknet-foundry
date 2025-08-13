#[starknet::interface]
trait IMetaTxV0Test<TContractState> {
    fn execute_meta_tx_get_caller(
        self: @TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_block_number(
        self: @TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_block_timestamp(
        self: @TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_sequencer_address(
        self: @TContractState,
        target: starknet::ContractAddress,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_get_block_hash(
        self: @TContractState,
        target: starknet::ContractAddress,
        block_number: u64,
        signature: Span<felt252>,
    ) -> felt252;
    fn execute_meta_tx_with_calldata(
        self: @TContractState,
        target: starknet::ContractAddress,
        entry_point_selector: felt252,
        calldata: Span<felt252>,
        signature: Span<felt252>,
    ) -> Span<felt252>;
    fn get_current_caller(self: @TContractState) -> felt252;
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
            self: @ContractState,
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
            self: @ContractState,
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
            self: @ContractState,
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
            self: @ContractState,
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
            self: @ContractState,
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

        fn execute_meta_tx_with_calldata(
            self: @ContractState,
            target: starknet::ContractAddress,
            entry_point_selector: felt252,
            calldata: Span<felt252>,
            signature: Span<felt252>,
        ) -> Span<felt252> {
            meta_tx_v0_syscall(target, entry_point_selector, calldata, signature)
                .unwrap()
        }

        fn get_current_caller(self: @ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }
    }
}
