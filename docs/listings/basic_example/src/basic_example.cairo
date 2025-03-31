use sncast_std::call;

// A real contract deployed on Sepolia network
const CONTRACT_ADDRESS: felt252 =
    0x07e867f1fa6da2108dd2b3d534f1fbec411c5ec9504eb3baa1e49c7a0bef5ab5;

fn main() {
    let call_result = call(
        CONTRACT_ADDRESS.try_into().unwrap(), selector!("get_greeting"), array![],
    )
        .expect('call failed');

    assert(*call_result.data[1] == 'Hello, Starknet!', *call_result.data[1]);

    println!("{:?}", call_result);
}
