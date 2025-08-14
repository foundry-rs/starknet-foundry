use attributes::fuzzer;
use attributes::{
    available_gas::available_gas, disable_predeployed_contracts::disable_predeployed_contracts,
    fork::fork, fuzzer::fuzzer, ignore::ignore,
    internal_config_statement::internal_config_statement, should_panic::should_panic, test::test,
};
use cairo_lang_macro::{ProcMacroResult, TokenStream, attribute_macro, executable_attribute};

mod args;
mod asserts;
pub mod attributes;
mod cairo_expression;
mod common;
mod config_statement;
mod parse;
mod types;
mod utils;

pub use utils::create_single_token;

executable_attribute!("snforge_internal_test_executable");

#[attribute_macro]
fn __internal_config_statement(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    internal_config_statement(args, item)
}

#[attribute_macro]
fn __fuzzer_config(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    fuzzer::fuzzer_config(args, item)
}

#[attribute_macro]
fn __fuzzer_wrapper(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    fuzzer::wrapper::fuzzer_wrapper(args, item)
}

#[attribute_macro]
fn param_test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    attributes::test_case::param_test(args, item)
}

#[attribute_macro]
fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    attributes::test_case::test_case(args, item)
}

#[attribute_macro]
fn __param_test_wrapper(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    attributes::test_case::wrapper::param_test_wrapper(args, item)
}

#[attribute_macro]
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

#[attribute_macro]
fn disable_predeployed_contracts(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    disable_predeployed_contracts(args, item)
}
