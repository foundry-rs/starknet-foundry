use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettingsTrait,
};

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        9999999999999999999,
        99999999999999999999999999999999999999,
        9999999999999999999,
        99999999999999999999999999999999999999,
        9999999999999999999,
        99999999999999999999999999999999999999,
    );
    let declare_nonce = get_nonce('latest');
    let declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce)).unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InsufficientAccountBalance),
        ) == declare_result,
        'ohno',
    )
}
