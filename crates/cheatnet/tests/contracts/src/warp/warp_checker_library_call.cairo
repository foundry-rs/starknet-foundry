use starknet::ClassHash;

#[starknet::interface]
trait IWarpChecker<TContractState> {
    fn get_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait IWarpCheckerLibCall<TContractState> {
    fn get_block_timestamp_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
}

#[starknet::contract]
mod WarpCheckerLibCall {
    use super::{IWarpCheckerDispatcherTrait, IWarpCheckerLibraryDispatcher};
    use starknet::ClassHash;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IWarpCheckerLibCall of super::IWarpCheckerLibCall<ContractState> {
        fn get_block_timestamp_with_lib_call(
            ref self: ContractState, class_hash: ClassHash
        ) -> u64 {
            let warp_checker = IWarpCheckerLibraryDispatcher { class_hash };
            warp_checker.get_block_timestamp()
        }
    }
}
