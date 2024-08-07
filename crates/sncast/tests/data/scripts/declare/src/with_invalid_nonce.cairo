use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettings, EthFeeSettings
};

fn main() {
    let max_fee = 99999999999999999;

    let declare_nonce = get_nonce('pending') + 100;
    let declare_result = declare(
        "Mapa",
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::Some(max_fee) }),
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
