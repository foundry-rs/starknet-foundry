use core::array::ArrayTrait;
use core::result::ResultTrait;
use simple_package_with_cheats::{
    ICheatedConstructorDispatcher, ICheatedConstructorDispatcherTrait, IHelloStarknetDispatcher,
    IHelloStarknetDispatcherTrait, IHelloStarknetProxyDispatcher,
    IHelloStarknetProxyDispatcherTrait,
};
use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, declare, start_cheat_block_number_global};
use starknet::contract_address;

#[test]
fn call_and_invoke() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetDispatcher { contract_address };

    let block_number = dispatcher.get_block_number();
    println!("block number {}", block_number);
    // TODO investigate why the default is 2000
    assert(block_number == 2000, 'block_info == 2000');

    start_cheat_block_number_global(123);

    let block_number = dispatcher.get_block_number();
    assert(block_number == 123, 'block_info == 123');
}

#[test]
fn call_and_invoke_proxy() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();

    let proxy_contract = declare("HelloStarknetProxy").unwrap().contract_class();
    let mut constructor_calldata = ArrayTrait::new();
    contract_address.serialize(ref constructor_calldata);
    let (proxy_contract_address, _) = proxy_contract.deploy(@constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetProxyDispatcher { contract_address: proxy_contract_address };

    let block_number = dispatcher.get_block_number();
    assert(block_number == 2000, 'block_number == 2000');

    start_cheat_block_number_global(123);

    let block_number = dispatcher.get_block_number();
    assert(block_number == 123, 'block_number == 123');
}

#[test]
fn call_and_invoke_library_call() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();

    let proxy_contract = declare("HelloStarknetProxy").unwrap().contract_class();
    let mut constructor_calldata = ArrayTrait::new();
    contract_address.serialize(ref constructor_calldata);
    let (proxy_contract_address, _) = proxy_contract.deploy(@constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetProxyDispatcher { contract_address: proxy_contract_address };

    let block_number = dispatcher.get_block_number_library_call();
    assert(block_number == 2000, 'block_number == 2000');

    start_cheat_block_number_global(123);

    let block_number = dispatcher.get_block_number_library_call();
    assert(block_number == 123, 'block_number == 123');
}

#[test]
fn deploy_syscall() {
    let contract = declare("HelloStarknet").unwrap().contract_class();
    let constructor_calldata = @ArrayTrait::new();
    let (contract_address, _) = contract.deploy(constructor_calldata).unwrap();

    let proxy_contract = declare("HelloStarknetProxy").unwrap().contract_class();
    let mut constructor_calldata = ArrayTrait::new();
    contract_address.serialize(ref constructor_calldata);
    let (proxy_contract_address, _) = proxy_contract.deploy(@constructor_calldata).unwrap();
    let dispatcher = IHelloStarknetProxyDispatcher { contract_address: proxy_contract_address };

    let class_hash = declare("CheatedConstructor").unwrap().contract_class().class_hash;

    let contract_address = dispatcher.deploy_cheated_constructor_contract(*class_hash, 111);
    let cheated_constructor_dispatcher = ICheatedConstructorDispatcher { contract_address };
    let block_number = cheated_constructor_dispatcher.get_stored_block_number();
    assert(block_number == 2000, 'block_number == 2000');

    start_cheat_block_number_global(123);

    let contract_address = dispatcher.deploy_cheated_constructor_contract(*class_hash, 222);
    let cheated_constructor_dispatcher = ICheatedConstructorDispatcher { contract_address };
    let block_number = cheated_constructor_dispatcher.get_stored_block_number();
    assert(block_number == 123, 'block_number == 123');
}
