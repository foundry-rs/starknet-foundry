use sncast_std::{FeeSettingsTrait, declare};

fn main() {
    let fee_settings = FeeSettingsTrait::max_fee(9999999);

    let result = declare("HelloStarknet", fee_settings, Option::None).expect('declare failed');

    println!("declare result: {}", result);
    println!("debug declare result: {:?}", result);
}
