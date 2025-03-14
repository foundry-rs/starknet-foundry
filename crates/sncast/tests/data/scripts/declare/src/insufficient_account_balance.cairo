use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettings,
};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None,
        l1_gas: Option::Some(9999999999999999999),
        l1_gas_price: Option::Some(99999999999999999999999999999999999999),
        l2_gas: Option::Some(9999999999999999999),
        l2_gas_price: Option::Some(99999999999999999999999999999999999999),
        l1_data_gas: Option::Some(9999999999999999999),
        l2_data_gas_price: Option::Some(99999999999999999999999999999999999999),
    };
    let declare_nonce = get_nonce('latest');
    let declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce)).unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InsufficientAccountBalance)
        ) == declare_result,
        'ohno'
    )
}
