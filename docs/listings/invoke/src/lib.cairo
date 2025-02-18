use starknet::ContractAddress;
use sncast_std::{invoke, InvokeResult, FeeSettings};

fn main() {
    let contract_address: ContractAddress =
        0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');

    let result = invoke(
        contract_address,
        selector!("put"),
        array![0x1, 0x2],
        FeeSettings {
            max_fee: Option::None, max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::None
    )
        .expect('invoke failed');

    println!("invoke result: {}", result);
    println!("debug invoke result: {:?}", result);
}
