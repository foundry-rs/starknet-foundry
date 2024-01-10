use starknet::ContractAddress;
use openzeppelin::token::erc20::interface::IERC20;
use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

#[starknet::interface]
trait IMyContract<TContractState> {
    fn claim(ref self: TContractState, asset: IERC20Dispatcher, account: ContractAddress);
}

#[starknet::contract]
mod MyContract {
    use starknet::{ContractAddress, get_contract_address, get_execution_info};
    use openzeppelin::token::erc20::interface::{IERC20Dispatcher, IERC20DispatcherTrait};

    #[storage]
    struct Storage {}

    #[external(v0)]
    impl MyContractImpl of super::IMyContract<ContractState> {
        fn claim(ref self: ContractState, asset: IERC20Dispatcher, account: ContractAddress) {
            let balance = asset.balance_of(get_contract_address());
            let _ = asset.balance_of(account);
            let _ = asset.total_supply();
            let _ = asset.allowance(account, get_contract_address());
            asset.transfer(account, balance);
        }
    }
}
