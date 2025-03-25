use sncast_std::{declare, FeeSettings};

fn main() {
    let fee_settings = FeeSettings {
        max_fee: Option::None, max_gas: Option::None, max_gas_unit_price: Option::None
    };
    let _ = declare("whatever", fee_settings, Option::None);
}
