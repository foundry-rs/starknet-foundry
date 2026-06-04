#[starknet::interface]
pub trait IVault<TContractState> {
    fn deposit(ref self: TContractState, amount: u64);
    fn withdraw(ref self: TContractState, amount: u64);
    fn balance(self: @TContractState) -> u64;
}

#[starknet::contract]
pub mod Vault {
    use starknet::storage::{StoragePointerReadAccess, StoragePointerWriteAccess};

    #[storage]
    struct Storage {
        balance: u64,
    }

    #[abi(embed_v0)]
    impl VaultImpl of super::IVault<ContractState> {
        fn deposit(ref self: ContractState, amount: u64) {
            let prev = self.balance.read();
            self.balance.write(prev + amount);
        }

        fn withdraw(ref self: ContractState, amount: u64) {
            let prev = self.balance.read();
            assert(prev >= amount, 'insufficient balance');
            self.balance.write(prev - amount);
        }

        fn balance(self: @ContractState) -> u64 {
            self.balance.read()
        }
    }
}
