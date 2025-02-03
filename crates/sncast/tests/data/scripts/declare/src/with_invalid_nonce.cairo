use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings
};

fn main() {
    let max_fee = 99999999999999999;

    let declare_nonce = get_nonce('pending') + 100;
    let declare_result = declare(
        "Mapa",
        FeeSettings {
            max_fee: Option::Some(max_fee), max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::Some(declare_nonce)
    )
        .unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InvalidTransactionNonce)
        ) == declare_result,
        'ohno'
    )
}
