#[starknet::interface]
trait IPrankChecker<TContractState> {
    fn get_caller_address(ref self: TContractState) -> felt252;
}

#[starknet::contract]
mod DeployAtChecked {
    use starknet::ContractAddress;
    use traits::Into;

    #[external(v0)]
    fn get_caller_address(ref self: ContractState, contract_address: ContractAddress) -> felt252 {
        IPrankChecker{ contract_address }.get_caller_address
    }
}
