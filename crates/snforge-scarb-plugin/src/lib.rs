use crate::attributes::{
    available_gas::_available_gas, fork::_fork, fuzzer::_fuzzer, ignore::_ignore,
    should_panic::_should_panic, test::_test,
};
use cairo_lang_macro::{attribute_macro, Diagnostics, ProcMacroResult, TokenStream};

mod args;
mod asserts;
mod attributes;
mod config_fn;
mod parse;

type MacroResult = Result<TokenStream, Diagnostics>;

#[attribute_macro]
fn test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    as_proc_macro_result(item.clone(), _test(args, item))
}

#[attribute_macro]
fn ignore(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    as_proc_macro_result(item.clone(), _ignore(args, item))
}

#[attribute_macro]
fn fuzzer(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    as_proc_macro_result(item.clone(), _fuzzer(args, item))
}

#[attribute_macro]
fn fork(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    as_proc_macro_result(item.clone(), _fork(args, item))
}

#[attribute_macro]
fn available_gas(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    as_proc_macro_result(item.clone(), _available_gas(args, item))
}

#[attribute_macro]
fn should_panic(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    as_proc_macro_result(item.clone(), _should_panic(args, item))
}

fn as_proc_macro_result(
    item: TokenStream,
    res: Result<TokenStream, impl Into<Diagnostics>>,
) -> ProcMacroResult {
    match res {
        Ok(item) => ProcMacroResult::new(item),
        Err(diagnostics) => ProcMacroResult::new(item).with_diagnostics(diagnostics.into()),
    }
}
