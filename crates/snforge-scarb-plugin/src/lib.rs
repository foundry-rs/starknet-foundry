use attributes::{
    available_gas::AvailableGasCollector, fork::ForkCollector, fuzzer::FuzzerCollector,
    ignore::IgnoreCollector, internal_config_statement::internal_config_statement,
    should_panic::ShouldPanicCollector, test::test,
};
use cairo_lang_macro::{attribute_macro, executable_attribute, ProcMacroResult, TokenStream};
use config_statement::extend_with_config_cheatcodes;

mod args;
mod asserts;
mod attributes;
mod config_statement;
mod parse;
mod utils;
mod validate;

executable_attribute!("test_executable");

#[attribute_macro]
#[allow(clippy::needless_pass_by_value)]
fn internal_config_statement(_args: TokenStream, item: TokenStream) -> ProcMacroResult {
    internal_config_statement(item)
}

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
