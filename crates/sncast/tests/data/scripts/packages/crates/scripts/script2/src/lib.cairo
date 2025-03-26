use sncast_std::{declare, FeeSettingsImpl};

fn main() {
    let fee_settings = FeeSettingsImpl::estimate();
    let _ = declare("whatever", fee_settings, Option::None);
}
