use sncast_std::{
    declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings,
};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::None,
        l1_gas_price: Option::None,
        l2_gas: Option::None,
        l2_gas_price: Option::None,
        l1_data_gas: Option::None,
        l2_data_gas_price: Option::None,
    };
    let declare_result = declare("Mapaaaa", fee_settings, Option::None).unwrap_err();
    println!("{:?}", declare_result);
}

