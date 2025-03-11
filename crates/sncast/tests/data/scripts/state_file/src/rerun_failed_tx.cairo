use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce,
    FeeSettings
};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::Some(1000000),
        l1_gas_price: Option::Some(10000000000000),
        l2_gas: Option::Some(1000000000),
        l2_gas_price: Option::Some(100000000000000000),
        l1_data_gas: Option::Some(1000000),
        l2_data_gas_price: Option::Some(10000000000000),
    };
    let map_contract_address = 0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
        .try_into()
        .expect('Invalid contract address value');

    invoke(map_contract_address, selector!("put"), array![0x10, 0x1], fee_settings, Option::None)
        .unwrap();
}
