#[starknet::interface]
pub trait IShaChecker<T> {
    fn use_sha(ref self: T);
}

#[starknet::contract]
mod ShaChecker {
    use super::IShaChecker;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IShaCheckerImpl of IShaChecker<ContractState> {
        fn use_sha(ref self: ContractState) {
            // sha256_process_block syscall
            let _sha256 = core::sha256::compute_sha256_u32_array(array![0x68656c6c], 0x6f, 1);
            // sha512_process_block syscall
            let _sha512 = core::sha512::compute_sha512_u64_array(
                array![0x48656c6c6f20776f], 0x726c64, 3,
            );
        }
    }
}
