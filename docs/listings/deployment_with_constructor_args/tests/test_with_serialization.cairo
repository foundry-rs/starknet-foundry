use deployment_with_constructor_args::Product;
use snforge_std::{ContractClassTrait, DeclareResult, DeclareResultTrait, declare};
use starknet::storage::StorableStoragePointerReadAccess;


#[test]
fn test_initial_cart_non_empty_with_serialization() {
    // 1. Declare contract
    let declare_result: DeclareResult = declare("ShoppingCart").unwrap();
    let contract = declare_result.contract_class();

    // 2. Create deployment parameters
    let initial_products = array![
        Product { name: 'Bread', price: 5, quantity: 2 },
        Product { name: 'Milk', price: 2, quantity: 4 },
        Product { name: 'Eggs', price: 3, quantity: 12 },
    ];

    // 3. Create calldata
    let mut calldata = ArrayTrait::new();

    // 4. Serialize initial products
    initial_products.serialize(ref calldata);

    // 5. Deploy the contract
    let (_contract_address, _) = contract.deploy(@calldata).unwrap();
}
