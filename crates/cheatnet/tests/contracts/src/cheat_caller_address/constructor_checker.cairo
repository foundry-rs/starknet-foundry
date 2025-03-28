use starknet::ContractAddress;

#[starknet::interface]
trait IConstructorCheatCallerAddressChecker<TContractState> {
    fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
    fn get_caller_address(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod ConstructorCheatCallerAddressChecker {
    use starknet::ContractAddress;

    #[storage]
    struct Storage {
        caller_address: ContractAddress,
    }

    #[constructor]
    fn constructor(ref self: ContractState) {
        let address = starknet::get_caller_address();
        self.caller_address.write(address);
    }

    #[abi(embed_v0)]
    impl IConstructorCheatCallerAddressChecker of super::IConstructorCheatCallerAddressChecker<
        ContractState,
    > {
        fn get_stored_caller_address(ref self: ContractState) -> ContractAddress {
            self.caller_address.read()
        }

        fn get_caller_address(ref self: ContractState) -> felt252 {
            starknet::get_caller_address().into()
        }
    }
}
