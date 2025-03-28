use core::array::ArrayTrait;
use core::result::ResultTrait;

use snforge_std::{declare, ContractClassTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;

use only_integration::IHelloStarknetDispatcher;
use only_integration::IHelloStarknetDispatcherTrait;

#[test]
fn declare_and_call_contract_from_lib() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance != 100');
}
