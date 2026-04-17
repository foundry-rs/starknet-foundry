use snforge_std::{Token, TokenTrait};

#[starknet::interface]
trait IERC20<TContractState> {
    fn decimals(self: @TContractState) -> u8;
}

#[test]
fn test_decimals() {
    let dispatcher = IERC20Dispatcher { contract_address: Token::STRK.contract_address() };
    dispatcher.decimals();
}
