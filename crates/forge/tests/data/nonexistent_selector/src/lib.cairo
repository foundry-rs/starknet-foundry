#[starknet::interface]
pub trait IMyContract<TState> {
    fn my_function(self: @TState);
}

#[starknet::contract]
pub mod MyContract {
    use starknet::SyscallResultTrait;
    use starknet::syscalls::call_contract_syscall;

    #[storage]
    pub struct Storage {}

    #[abi(embed_v0)]
    impl MyContract of super::IMyContract<ContractState> {
        fn my_function(self: @ContractState) {
            let this = starknet::get_contract_address();
            let selector = selector!("nonexistent");
            let calldata = array![].span();

            call_contract_syscall(this, selector, calldata).unwrap_syscall();
        }
    }
}
