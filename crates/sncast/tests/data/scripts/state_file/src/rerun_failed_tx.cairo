use sncast_std::{invoke, FeeSettings};

fn main() {
    let map_contract_address = 0xcd8f9ab31324bb93251837e4efb4223ee195454f6304fcfcb277e277653008
        .try_into()
        .expect('Invalid contract address value');

    invoke(
        map_contract_address,
        selector!("put"),
        array![0x10, 0x1],
        FeeSettings {
            max_fee: Option::None, max_gas: Option::None, max_gas_unit_price: Option::None
        },
        Option::None
    )
        .unwrap();
}
