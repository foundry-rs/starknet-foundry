use snforge_std::{ContractClassTrait, DeclareResult, DeclareResultTrait, declare};
use testing_smart_contracts_constructor_params::{
    IShoppingCartDispatcher, IShoppingCartDispatcherTrait, Product,
};

#[test]
fn test_initial_cart_non_empty() {
    let initial_products = array![
        Product { name: 'Bread', price: 5, quantity: 2 },
        Product { name: 'Milk', price: 2, quantity: 4 },
        Product { name: 'Eggs', price: 3, quantity: 12 },
    ];
    let declare_result: DeclareResult = declare("ShoppingCart").unwrap();

    let class_hash = declare_result.contract_class().class_hash;
    let contract_deployer = ShoppingCartDeployer {
        class_hash: *class_hash, contract_address_salt: 0, deploy_from_zero: true,
    };

    let (contract_address, _) = contract_deployer.deploy(initial_products).unwrap();

    let dispatcher = IShoppingCartDispatcher { contract_address };

    let products = dispatcher.get_products();
    assert(products.len() == 3, 'Invalid products count');
}
