use sncast_std::{
    get_nonce, declare, ScriptCommandError, ProviderError, StarknetError, FeeSettingsTrait,
};

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(1, 1, 1, 1, 1, 1);
    let declare_nonce = get_nonce('latest');
    let declare_result = declare("Mapa", fee_settings, Option::Some(declare_nonce)).unwrap_err();
    println!("{:?}", declare_result);

    assert(
        ScriptCommandError::ProviderError(
            ProviderError::StarknetError(StarknetError::InsufficientResourcesForValidate),
        ) == declare_result,
        'ohno',
    )
}
