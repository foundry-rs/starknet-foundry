use sncast_std::{
    declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings,
    EthFeeSettings
};

fn main() {
    let declare_result = declare(
        "Mapaaaa", FeeSettings::Eth(EthFeeSettings { max_fee: Option::None }), Option::None
    )
        .unwrap_err();
    println!("{:?}", declare_result);
}

