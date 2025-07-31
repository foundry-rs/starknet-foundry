use snforge_std::cheatcodes::contract_class::DeclareResultTrait;
use snforge_std::{declare, ContractClassTrait, L1HandlerTrait};

#[test]
fn test_l1_handler() {
    let example_contract = declare("L1HandlerExample").unwrap().contract_class();

    // Deploy the target Starknet contract
    let (contract_address, _) = example_contract.deploy(@array![]).unwrap();

    // Define the L1 handler to be called
    let l1_handler = L1HandlerTrait::new(contract_address, selector!("handle_l1_message"));

    let eth_address = 0x123; // Ethereum address of the message sender

    // The payload to be sent to the L1 handler, serialized with `Serde`
    // 3 is the number of array and 1, 2, 3 are the actual elements of the array
    let payload = array![3, 1, 2, 3];

    // Execute the L1 handler with the Ethereum address and payload
    l1_handler.execute(eth_address, payload.span()).unwrap();
}
