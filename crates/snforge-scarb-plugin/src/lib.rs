use crate::attributes::test::test;
use attributes::{
    available_gas::AvailableGasCollector, fork::ForkCollector, fuzzer::FuzzerCollector,
    ignore::IgnoreCollector, should_panic::ShouldPanicCollector,
};
use cairo_lang_macro::{attribute_macro, executable_attribute, ProcMacroResult, TokenStream};
use config_statement::extend_with_config_cheatcodes;

mod args;
mod asserts;
mod attributes;
mod config_statement;
mod parse;

executable_attribute!("test_executable");

#[attribute_macro]
#[allow(clippy::needless_pass_by_value)]
fn test(_args: TokenStream, item: TokenStream) -> ProcMacroResult {
    test(item)
}

#[attribute_macro]
fn ignore(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<IgnoreCollector>(args, item)
}

#[attribute_macro]
fn fuzzer(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<FuzzerCollector>(args, item)
}

#[attribute_macro]
fn fork(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<ForkCollector>(args, item)
}

#[attribute_macro]
fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<AvailableGasCollector>(args, item)
}

#[attribute_macro]
fn should_panic(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<ShouldPanicCollector>(args, item)
}
