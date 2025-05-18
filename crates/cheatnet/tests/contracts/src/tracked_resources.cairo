// Address of a simple proxy contract that works as a wrapper on the `call_contract_syscall`
// Deployed to the Sepolia network and compiled with Sierra version 1.6.0
const PROXY_CONTRACT_ADDRESS: felt252 =
    0x004a053601baaed3231638627631caed753b6527484cde2ed2b5b7d57854a902;

#[starknet::contract]
mod TrackedResources {
    use starknet::{syscalls, SyscallResultTrait, get_contract_address};

    #[storage]
    struct Storage {}

    #[external(v0)]
    fn call_twice(ref self: ContractState) {
        // Call through proxy
        syscalls::call_contract_syscall(
            super::PROXY_CONTRACT_ADDRESS.try_into().unwrap(),
            selector!("call_single"),
            array![get_contract_address().try_into().unwrap(), selector!("call_internal"), 0]
                .span(),
        )
            .unwrap_syscall();

        // Call itself directly
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
