use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings
};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::Some(100000),
        l1_gas_price: Option::Some(10000000000000),
        l2_gas: Option::Some(1000000000),
        l2_gas_price: Option::Some(100000000000000000000),
        l1_data_gas: Option::Some(100000),
        l2_data_gas_price: Option::Some(10000000000000),
    };
    let declare_nonce = get_nonce('latest');
    let declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce)).unwrap_err();

    println!("{:?}", declare_result);
}
