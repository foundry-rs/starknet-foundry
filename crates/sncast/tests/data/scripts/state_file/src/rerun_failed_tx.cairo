use sncast_std::{
    declare, deploy, invoke, call, DeclareResult, DeployResult, InvokeResult, CallResult, get_nonce,
    FeeSettings, EthFeeSettings
};

fn main() {
    let map_contract_address = 0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
        .try_into()
        .expect('Invalid contract address value');

    invoke(
        map_contract_address,
        selector!("put"),
        Option::Some("{0x10, 0x1}"),
        FeeSettings::Eth(EthFeeSettings { max_fee: Option::None }),
        Option::None
    )
        .unwrap();
}
