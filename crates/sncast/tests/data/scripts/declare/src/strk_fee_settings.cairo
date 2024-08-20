use sncast_std::{
    get_nonce, declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError,
    FeeSettings, StrkFeeSettings
};

fn main() {
    let declare_nonce = get_nonce('latest');
    declare(
        "Mapa",
        FeeSettings::Strk(
            StrkFeeSettings {
                max_gas: Option::Some(99999),
                max_gas_unit_price: Option::Some(999999999999),
                max_fee: Option::None
            }
        ),
        Option::Some(declare_nonce)
    )
        .expect('declare failed');
    println!("success");
}
