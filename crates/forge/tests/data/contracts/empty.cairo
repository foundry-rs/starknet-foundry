#[starknet::interface]
trait IHelloStarknet<TContractState> {
    fn increase_balance(ref self: TContractState);
    fn increase_balance_empty(ref self: TContractState);
}

#[starknet::contract]
mod HelloEmpty {
    use starknet::syscalls::keccak_syscall;
    use starknet::SyscallResultTrait;

    #[storage]
    struct Storage {
    }

    #[external(v0)]
    impl IHelloStarknetImpl of super::IHelloStarknet<ContractState> {
        fn increase_balance(ref self: ContractState) {
            keccak_syscall(array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17].span()).unwrap_syscall();
        }

        fn increase_balance_empty(ref self: ContractState) {}
    }
}
