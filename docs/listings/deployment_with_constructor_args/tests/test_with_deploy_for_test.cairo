use snforge_std::{DeclareResult, DeclareResultTrait, declare};
use starknet::deployment::DeploymentParams;
use starknet::storage::StorableStoragePointerReadAccess;
use deployment_with_constructor_args::Product;
use deployment_with_constructor_args::ShoppingCart::deploy_for_test;

#[test]
fn test_initial_cart_non_empty_with_deploy_for_test() {
    // 1. Declare contract
    let declare_result: DeclareResult = declare("ShoppingCart").unwrap();
    let class_hash = declare_result.contract_class().class_hash;

    // 2. Create deployment parameters
    let deployment_params = DeploymentParams { salt: 0, deploy_from_zero: true };
    // 3. Create initial products
    let initial_products = array![
        Product { name: 'Bread', price: 5, quantity: 2 },
        Product { name: 'Milk', price: 2, quantity: 4 },
        Product { name: 'Eggs', price: 3, quantity: 12 },
    ];

    // 4. Use `deploy_for_test` to deploy the contract
    // It automatically handles serialization of constructor parameters
    let (_contract_address, _) = deploy_for_test(*class_hash, deployment_params, initial_products)
        .expect('Deployment failed');
}
