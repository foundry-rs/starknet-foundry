use attributes::{
    available_gas::available_gas, fork::fork, fuzzer::fuzzer, ignore::ignore,
    internal_config_statement::internal_config_statement, should_panic::should_panic, test::test,
};
use cairo_lang_macro::{attribute_macro, executable_attribute, ProcMacroResult, TokenStream};

mod args;
mod asserts;
pub mod attributes;
mod cairo_expression;
mod common;
mod config_statement;
mod parse;
mod types;
mod utils;

executable_attribute!("snforge_internal_test_executable");

#[attribute_macro]
#[allow(clippy::needless_pass_by_value)]
fn __internal_config_statement(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    internal_config_statement(args, item)
}

#[attribute_macro]
#[allow(clippy::needless_pass_by_value)]
fn test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    test(args, item)
}

#[attribute_macro]
fn ignore(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    ignore(args, item)
}

#[attribute_macro]
fn fuzzer(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    fuzzer(args, item)
}

#[attribute_macro]
fn fork(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    fork(args, item)
}

#[attribute_macro]
fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    available_gas(args, item)
}

#[attribute_macro]
fn should_panic(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    should_panic(args, item)
}
