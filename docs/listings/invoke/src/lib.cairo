use starknet::ContractAddress;
use sncast_std::{invoke, FeeSettings};

fn main() {
    let contract_address: ContractAddress =
        0x1e52f6ebc3e594d2a6dc2a0d7d193cb50144cfdfb7fdd9519135c29b67e427
        .try_into()
        .expect('Invalid contract address value');
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::None,
        l1_gas_price: Option::None,
        l2_gas: Option::None,
        l2_gas_price: Option::None,
        l1_data_gas: Option::None,
        l2_data_gas_price: Option::None,
    };

    let result = invoke(
        contract_address, selector!("put"), array![0x1, 0x2], fee_settings, Option::None
    )
        .expect('invoke failed');

    println!("invoke result: {}", result);
    println!("debug invoke result: {:?}", result);
}
