#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState, amount: felt252);
    fn get_balance(self: @TContractState) -> felt252;
    fn get_block_number(self: @TContractState) -> u64;
}

#[starknet::interface]
pub trait IHelloStarknetProxy<TContractState> {
    fn get_block_number(self: @TContractState) -> u64;
}

#[starknet::contract]
pub mod HelloStarknetProxy {
    use starknet::ContractAddress;
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};
    use crate::{IHelloStarknetDispatcher, IHelloStarknetDispatcherTrait};

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
