use sncast_std::{
    invoke, InvokeResult, ScriptCommandError, ProviderError, StarknetError, FeeSettingsTrait,
};

fn main() {
    let map_contract_address = 0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
        .try_into()
        .expect('Invalid contract address value');
    let fee_settings = FeeSettingsTrait::estimate();

    let invoke_result = invoke(
        map_contract_address, selector!("mariusz"), array![0x10, 0x1], fee_settings, Option::None,
    )
        .unwrap_err();
    println!("{:?}", invoke_result);
}

