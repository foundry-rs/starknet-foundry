use sncast_std::{declare, FeeSettingsTrait};

fn main() {
    let fee_settings = FeeSettingsTrait::estimate();
    let declare_result = declare("Mapaaaa", fee_settings, Option::None).unwrap_err();
    println!("{:?}", declare_result);
}

