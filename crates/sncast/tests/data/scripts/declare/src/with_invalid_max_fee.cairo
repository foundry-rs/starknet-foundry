use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettings
};

fn main() {
    let declare_nonce = get_nonce('latest');
    let declare_result = declare(
        "Mapa",
        FeeSettings { max_fee: Option::None, max_gas: Option::Some(1), max_gas_unit_price: Option::Some(1) },
        Option::Some(declare_nonce)
    )
        .unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InsufficientMaxFee)
        ) == declare_result,
        'ohno'
    )
}
