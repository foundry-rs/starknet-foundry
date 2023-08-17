#[starknet::interface]
trait IHelloKeccak<TContractState> {
    fn run_keccak(ref self: TContractState) -> u256;
}

#[starknet::contract]
mod HelloKeccak {
    use array::ArrayTrait;
    use starknet::syscalls::keccak_syscall;

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl IHelloKeccakImpl of super::IHelloKeccak<ContractState> {
        fn run_keccak(ref self: ContractState) -> u256 {
            let input = array![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17];
            keccak_syscall(input.span()).unwrap_syscall()
        }
    }
}
