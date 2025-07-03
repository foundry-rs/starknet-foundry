#[starknet::interface]
pub trait IHelloStarknet<TContractState> {
    fn example_function(ref self: TContractState);
}

#[starknet::contract]
pub mod HelloStarknet {
    use starknet::SyscallResultTrait;
    use core::sha256::compute_sha256_u32_array;
    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl HelloStarknetImpl of super::IHelloStarknet<ContractState> {
        fn example_function(ref self: ContractState) {
            core::pedersen::pedersen(1, 2);
            core::keccak::keccak_u256s_le_inputs(array![1].span());
            let _hash = compute_sha256_u32_array(array![0x68656c6c], 0x6f, 1);
            starknet::syscalls::get_block_hash_syscall(1).unwrap_syscall();
            starknet::syscalls::emit_event_syscall(array![1].span(), array![2].span())
                .unwrap_syscall();
        }
    }
}
