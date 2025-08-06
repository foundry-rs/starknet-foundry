use cheatcodes_reference::mock_call_example::{
    IShoppingCartDispatcher, IShoppingCartDispatcherTrait, Product,
};
use snforge_std::{ContractClassTrait, DeclareResultTrait, declare, start_mock_call, stop_mock_call};

#[test]
fn test_mock_call() {
    // 1. Create calldata for the contract deployment
    let mut calldata = ArrayTrait::new();

    // 2. Serialize intial products (in this case an empty array)
    let initial_products: Array<Product> = array![];
    initial_products.serialize(ref calldata);

    // 3. Declare and deploy the ShoppingCart contract
    let contract = declare("ShoppingCart").unwrap().contract_class();
    let (contract_address, _) = contract.deploy(@calldata).unwrap();

    // 4. Create an instance of the dispatcher
    let dispatcher = IShoppingCartDispatcher { contract_address };

    // 5. Mock the returned value of the `get_products` function
    let mock_products = array![
        Product { name: 'Banana', price: 2, quantity: 5 },
        Product { name: 'Apple', price: 3, quantity: 10 },
    ];

    // 6. Start the mock call with the mocked products
    start_mock_call(contract_address, selector!("get_products"), mock_products);

    // 7. Call the `get_products` function through the dispatcher
    let retrieved_products: Array<Product> = dispatcher.get_products();

    // 8. Verify the retrieved products
    let product_0 = retrieved_products.at(0);
    assert(*product_0.name == 'Banana', 'product_0.name');
    assert(*product_0.price == 2, 'product_0.price');
    assert(*product_0.quantity == 5, 'product_0.quantity');

    let product_1 = retrieved_products.at(1);
    assert(*product_1.name == 'Apple', 'product_1.name');
    assert(*product_1.price == 3, 'product_1.price');
    assert(*product_1.quantity == 10, 'product_1.quantity');

    // 9. Stop the mock call
    stop_mock_call(contract_address, selector!("get_products"));
}
