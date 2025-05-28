use sncast_std::{
    DeclareResult, FeeSettingsTrait, ProviderError, ScriptCommandError, StarknetError, declare,
};

fn main() {
    let fee_settings = FeeSettingsTrait::estimate();
    let declare_result = declare("Mapaaaa", fee_settings, Option::None).unwrap_err();
    println!("{:?}", declare_result);
}

