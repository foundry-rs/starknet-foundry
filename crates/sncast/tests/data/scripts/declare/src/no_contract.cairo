use sncast_std::{
    declare, DeclareResult, ScriptCommandError, ProviderError, StarknetError, FeeSettings,
};

fn main() {
    let declare_result = declare(
        "Mapaaaa", FeeSettings { max_fee: Option::None, max_gas: Option::None, max_gas_unit_price: Option::None }, Option::None
    )
        .unwrap_err();
    println!("{:?}", declare_result);
}

