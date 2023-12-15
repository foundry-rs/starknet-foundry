use starknet::ContractAddress;

#[starknet::interface]
trait IConstructorPrankChecker<TContractState> {
    fn get_stored_caller_address(ref self: TContractState) -> ContractAddress;
}

#[starknet::contract]
mod ConstructorPrankChecker {
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
    impl IConstructorPrankChecker of super::IConstructorPrankChecker<ContractState> {
        fn get_stored_caller_address(ref self: ContractState) -> ContractAddress {
            self.caller_address.read()
        }
    }
}
