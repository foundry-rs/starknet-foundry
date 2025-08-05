use sncast_std::{FeeSettingsTrait, declare};

fn main() {
    let fee_settings = FeeSettingsTrait::estimate();
    let declare_result = declare("Mapaaaa", fee_settings, Option::None).unwrap_err();
    println!("{:?}", declare_result);
}

