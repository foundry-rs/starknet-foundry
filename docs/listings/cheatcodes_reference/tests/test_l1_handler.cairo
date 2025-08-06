use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{ContractClassTrait, L1HandlerTrait, declare};

#[test]
fn test_l1_handler() {
    // 1. Declare and deploy the Starknet contract
    let example_contract = declare("L1HandlerExample").unwrap().contract_class();
    let (contract_address, _) = example_contract.deploy(@array![]).unwrap();

    // 2. Define the L1 handler to be called
    let l1_handler = L1HandlerTrait::new(contract_address, selector!("handle_l1_message"));

    // 3. Define Ethereum address of the message sender
    let eth_address = 0x123;

    // 4. The payload to be sent to the L1 handler
    let payload = array![1, 2, 3];
    let mut serialized_payload = array![];
    payload.serialize(ref serialized_payload);

    // 5. Execute the L1 handler with the Ethereum address and payload
    // This will trigger the `handle_l1_message` function of the contract
    l1_handler.execute(eth_address, serialized_payload.span()).unwrap();
}
