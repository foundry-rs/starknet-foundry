use snforge_std::cheatcodes::erc20::{Token, TokenTrait};
use starknet::ContractAddress;

#[starknet::interface]
trait IERC20<TContractState> {
    fn balance_of(self: @TContractState, account: ContractAddress) -> u256;
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 828912)]
fn test_balance_of_strk() {
    let dispatcher = IERC20Dispatcher { contract_address: Token::STRK.contract_address() };
    dispatcher.balance_of(0x1234.try_into().unwrap());
}

#[test]
#[fork(url: "{{ NODE_RPC_URL }}", block_number: 828912)]
fn test_balance_of_eth() {
    let dispatcher = IERC20Dispatcher { contract_address: Token::ETH.contract_address() };
    dispatcher.balance_of(0x1234.try_into().unwrap());
}
