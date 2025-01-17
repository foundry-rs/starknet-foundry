use anyhow::Error;
use clap::ValueEnum;
use shared::print::print_as_warning;

const DEPRECATION_MESSAGE: &str = "The '--version' flag is deprecated and will be removed in the future. Version 3 will become the only type of transaction available.";

pub fn parse_version<T: ValueEnum>(s: &str) -> Result<T, String> {
    print_as_warning(&Error::msg(DEPRECATION_MESSAGE));
    T::from_str(s, true)
}
