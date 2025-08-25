use starknet::{ClassHash, ContractAddress};

#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn get_block_number(self: @TContractState) -> u64;
}

#[starknet::interface]
pub trait ICheatedConstructor<TContractState> {
    fn get_stored_block_number(self: @TContractState) -> u64;
}

#[starknet::contract]
pub mod CheatedConstructor {
    use starknet::get_block_number;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        block_number: u64,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let block_number = get_block_number();
        self.block_number.write(block_number);
    }

    #[abi(embed_v0)]
    impl ICheatedConstructorImpl of super::ICheatedConstructor<ContractState> {
        fn get_stored_block_number(self: @ContractState) -> u64 {
            self.block_number.read()
        }
    }
}

#[starknet::interface]
pub trait IHelloStarknetProxy<TContractState> {
    fn get_block_number(self: @TContractState) -> u64;
    fn get_block_number_library_call(self: @TContractState) -> u64;
    fn deploy_cheated_constructor_contract(
        ref self: TContractState, class_hash: ClassHash, salt: felt252,
    ) -> ContractAddress;
}

#[starknet::contract]
pub mod HelloStarknetProxy {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use starknet::syscalls::{deploy_syscall, get_class_hash_at_syscall};
    use starknet::{ClassHash, ContractAddress};
    use crate::{
        IHelloStarknetDispatcher, IHelloStarknetDispatcherTrait, IHelloStarknetLibraryDispatcher,
    };

    #[storage]
    struct Storage {
        address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState, address: ContractAddress) {
        self.address.write(address);
    }

    #[abi(embed_v0)]
    impl IHelloStarknetProxyImpl of super::IHelloStarknetProxy<ContractState> {
        fn get_block_number(self: @ContractState) -> u64 {
            let address = self.address.read();
            let proxied = IHelloStarknetDispatcher { contract_address: address };
            proxied.get_block_number()
        }

        fn get_block_number_library_call(self: @ContractState) -> u64 {
            let address = self.address.read();
            let class_hash = get_class_hash_at_syscall(address).unwrap();
            let library_dispatcher = IHelloStarknetLibraryDispatcher { class_hash };
            library_dispatcher.get_block_number()
        }

        fn deploy_cheated_constructor_contract(
            ref self: ContractState, class_hash: ClassHash, salt: felt252,
        ) -> ContractAddress {
            let (address, _) = deploy_syscall(class_hash, salt, array![].span(), false).unwrap();
            address
        }
    }
}

#[starknet::contract]
pub mod HelloStarknet {
    use core::array::ArrayTrait;
    use starknet::get_block_number;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        balance: felt252,
    }

    #[abi(embed_v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {
        // Increases the balance by the given amount
        fn increase_balance(ref self: ContractState, amount: felt252) {
            self.balance.write(self.balance.read() + amount);
        }

        // Returns the current balance
        fn get_balance(self: @ContractState) -> felt252 {
            self.balance.read()
        }

        fn get_block_number(self: @ContractState) -> u64 {
            get_block_number()
        }
    }
}
