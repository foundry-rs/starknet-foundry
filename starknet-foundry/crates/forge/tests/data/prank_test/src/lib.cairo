
#[starknet::contract]
mod PrankedContract {
    use starknet::get_caller_address;
    use starknet::ContractAddress;
    use starknet::ContractAddressIntoFelt252;
    use option::Option;
    use traits::Into;

    #[storage]
    struct Storage {
        a: felt252,
    }

    #[view]
    fn return_callers_address() -> felt252 {
        let caller_address: ContractAddress = get_caller_address();
        caller_address.into()
    }
}

