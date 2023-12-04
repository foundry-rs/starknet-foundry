#[starknet::interface]
trait IHelloKeccak<TContractState> {
    fn run_keccak(ref self: TContractState, input: Array<u64>) -> u256;
}

#[starknet::contract]
mod HelloKeccak {
    use array::ArrayTrait;
    use starknet::syscalls::keccak_syscall;
    use starknet::SyscallResultTrait;

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl IHelloKeccakImpl of super::IHelloKeccak<ContractState> {
        fn run_keccak(ref self: ContractState, input: Array<u64>) -> u256 {
            keccak_syscall(input.span()).unwrap_syscall()
        }
    }
}
