use sncast_std::call;

// Some nonexistent contract
const CONTRACT_ADDRESS: felt252 = 0x2137;

fn main() {
    // This call fails
    let call_result = call(
        CONTRACT_ADDRESS.try_into().unwrap(), selector!("get_greeting"), array![]
    );

    // Make some assertion
    assert!(call_result.is_err());

    // Print the result error
    println!("Received error: {:?}", call_result.unwrap_err());
}
