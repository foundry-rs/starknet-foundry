use core::array::ArrayTrait;
use core::result::ResultTrait;
pub use core::starknet::contract_address;

use snforge_std::{declare, ContractClassTrait};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;

pub use simple_package::hello_starknet::IHelloStarknetDispatcher;
pub use simple_package::hello_starknet::IHelloStarknetDispatcherTrait;

#[test]
fn call_and_invoke() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let balance = dispatcher.get_balance();
    assert(balance == 0, 'balance == 0');

    dispatcher.increase_balance(100);

    let balance = dispatcher.get_balance();
    assert(balance == 100, 'balance == 100');
}
