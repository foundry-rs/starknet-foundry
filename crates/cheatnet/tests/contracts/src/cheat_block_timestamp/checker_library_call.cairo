use starknet::ClassHash;

#[starknet::interface]
trait ICheatBlockTimestampChecker<TContractState> {
    fn get_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::interface]
trait ICheatBlockTimestampCheckerLibCall<TContractState> {
    fn get_block_timestamp_with_lib_call(ref self: TContractState, class_hash: ClassHash) -> u64;
    fn get_block_timestamp(ref self: TContractState) -> u64;
}

#[starknet::contract]
mod CheatBlockTimestampCheckerLibCall {
    use starknet::ClassHash;
    use super::{
        ICheatBlockTimestampCheckerDispatcherTrait, ICheatBlockTimestampCheckerLibraryDispatcher,
    };

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl ICheatBlockTimestampCheckerLibCall of super::ICheatBlockTimestampCheckerLibCall<
        ContractState,
    > {
        fn get_block_timestamp_with_lib_call(
            ref self: ContractState, class_hash: ClassHash,
        ) -> u64 {
            let cheat_block_timestamp_checker = ICheatBlockTimestampCheckerLibraryDispatcher {
                class_hash,
            };
            cheat_block_timestamp_checker.get_block_timestamp()
        }

        fn get_block_timestamp(ref self: ContractState) -> u64 {
            starknet::get_block_info().unbox().block_timestamp
        }
    }
}
