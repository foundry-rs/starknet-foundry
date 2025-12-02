#[starknet::interface]
pub trait IExampleContract<T> {
    fn set_class_hash(ref self: T, class_hash: starknet::ClassHash);
    fn get_caller_address(ref self: T) -> starknet::ContractAddress;
}

// contract A
#[starknet::contract]
pub mod ExampleContract {
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IExampleContractImpl of super::IExampleContract<ContractState> {
        fn set_class_hash(ref self: ContractState, class_hash: starknet::ClassHash) {}

        fn get_caller_address(ref self: ContractState) -> starknet::ContractAddress {
            starknet::get_caller_address()
        }
    }
}


// contract B to make library call to the class of contract A
#[starknet::contract]
pub mod ExampleContractLibraryCall {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use super::{IExampleContractDispatcherTrait, IExampleContractLibraryDispatcher};

    #[storage]
    struct Storage {
        lib_class_hash: starknet::ClassHash,
    }

    #[abi(embed_v0)]
    impl ExampleContract of super::IExampleContract<ContractState> {
        #[abi(embed_v0)]
        fn set_class_hash(ref self: ContractState, class_hash: starknet::ClassHash) {
            self.lib_class_hash.write(class_hash);
        }

        fn get_caller_address(ref self: ContractState) -> starknet::ContractAddress {
            IExampleContractLibraryDispatcher { class_hash: self.lib_class_hash.read() }
                .get_caller_address()
        }
    }
}
