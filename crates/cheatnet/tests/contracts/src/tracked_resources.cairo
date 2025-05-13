// Address of a map contract deployed to the Sepolia network
// Compiled with Sierra version 1.3.0
const SEPOLIA_MAP_ADDRESS: felt252 =
    0x06b248bde9ce00d69099304a527640bc9515a08f0b49e5168e2096656f207e1d;

#[starknet::contract]
mod TrackedResources {
    use starknet::{syscalls, SyscallResultTrait, get_contract_address};

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn call_contracts(ref self: ContractState) {
        syscalls::call_contract_syscall(
            super::SEPOLIA_MAP_ADDRESS.try_into().unwrap(),
            selector!("put"),
            array![0x100, 0x200].span(),
        )
            .unwrap_syscall();

        syscalls::call_contract_syscall(
            get_contract_address(), selector!("call_internal"), array![].span(),
        )
            .unwrap_syscall();
    }

    #[external(v0)]
    fn call_internal(ref self: ContractState) {
        dummy_computations();
    }

    fn dummy_computations() {
        1_u8 >= 1_u8;
        1_u8 & 1_u8;

        core::pedersen::pedersen(1, 2);
        core::poseidon::hades_permutation(0, 0, 0);
        core::keccak::keccak_u256s_le_inputs(array![1].span());

        syscalls::get_block_hash_syscall(0x100).unwrap_syscall();
        syscalls::emit_event_syscall(array![1].span(), array![2].span()).unwrap_syscall();
    }
}
