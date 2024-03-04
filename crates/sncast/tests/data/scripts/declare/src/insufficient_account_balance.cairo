use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError
};

fn main() {
    let max_fee = 9999999999999999999999999999999999;

    let declare_nonce = get_nonce('latest');
    let declare_result = declare("Mapa", Option::Some(max_fee), Option::Some(declare_nonce))
        .unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InsufficientAccountBalance)
        ) == declare_result,
        'ohno'
    )
}
