use core::array::ArrayTrait;
use should_panic_test::constructor_panic::{
    IConstructorPanickingContractProxyDispatcher, IConstructorPanickingContractProxyDispatcherTrait,
};
use snforge_std::{ContractClassTrait, DeclareResult, DeclareResultTrait, declare};

#[test]
#[should_panic]
fn deployment_with_panic_not_possible_to_catch() {
    let contract_class = declare("ConstructorPanickingContract").unwrap().contract_class();
    match contract_class.deploy(@array![]) {
        // Unreachable
        Result::Ok(_) => {},
        Result::Err(_) => {},
    }
}

#[test]
#[should_panic]
fn proxied_deployment_with_panic_not_possible_to_catch() {
    let proxy = declare("ConstructorPanickingContractProxy").unwrap().contract_class();
    let (proxy_address, _) = proxy.deploy(@array![]).unwrap();

    let dispatcher = IConstructorPanickingContractProxyDispatcher {
        contract_address: proxy_address,
    };

    let declare_result = declare("ConstructorPanickingContract").unwrap();
    let class_hash = match declare_result {
        DeclareResult::AlreadyDeclared(res) => res.class_hash,
        DeclareResult::Success(res) => res.class_hash,
    };

    match dispatcher.deploy_constructor_panicking_contract(class_hash) {
        // Unreachable
        Result::Ok(_) => {},
        Result::Err(_) => {},
    }
}

