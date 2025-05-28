use sncast_std::{
    DeclareResult, FeeSettingsTrait, ProviderError, ScriptCommandError, StarknetError, declare,
    get_nonce,
};

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let declare_nonce = get_nonce('latest');
    declare("Mapa", fee_settings, Option::Some(declare_nonce)).expect('declare failed');
    println!("success");
}
