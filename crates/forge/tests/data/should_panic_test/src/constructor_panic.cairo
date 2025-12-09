use starknet::{ClassHash, ContractAddress};

#[starknet::contract]
pub mod ConstructorPanickingContract {
    #[storage]
    struct Storage {}

    #[constructor]
    fn constructor(ref self: ContractState) {
        panic!("Panic message from constructor");
    }
}

#[starknet::interface]
pub trait IConstructorPanickingContractProxy<TContractState> {
    fn deploy_constructor_panicking_contract(
        self: @TContractState, class_hash: ClassHash,
    ) -> Result<ContractAddress, Span<felt252>>;
}

#[starknet::contract]
pub mod ConstructorPanickingContractProxy {
    use starknet::{ClassHash, ContractAddress};

    #[storage]
    struct Storage {}

    #[abi(embed_v0)]
    impl Impl of super::IConstructorPanickingContractProxy<ContractState> {
        fn deploy_constructor_panicking_contract(
            self: @ContractState, class_hash: ClassHash,
        ) -> Result<ContractAddress, Span<felt252>> {
            let deploy_result = starknet::syscalls::deploy_syscall(
                class_hash, 0, array![].span(), false,
            );
            match deploy_result {
                Result::Ok((contract_address, _)) => Result::Ok(contract_address),
                Result::Err(err) => Result::Err(err.span()),
            }
        }
    }
}
