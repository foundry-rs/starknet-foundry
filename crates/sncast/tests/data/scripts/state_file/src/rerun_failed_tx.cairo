use sncast_std::{invoke, FeeSettingsTrait};

fn main() {
    let fee_settings = FeeSettingsTrait::resource_bounds(
        100000, 10000000000000, 1000000000, 100000000000000000000, 100000, 10000000000000,
    );
    let map_contract_address = 0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
        .try_into()
        .expect('Invalid contract address value');

    invoke(map_contract_address, selector!("put"), array![0x10, 0x1], fee_settings, Option::None)
        .unwrap();
}
