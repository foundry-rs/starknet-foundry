use std::ops::Not;

use crate::{
    args::Arguments,
    asserts::assert_is_used_once,
    attributes::{
        AttributeInfo,
        fuzzer::{FuzzerCollector, FuzzerConfigCollector, wrapper::FuzzerWrapperCollector},
    },
    parse::{parse, parse_args},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{TypedSyntaxNode, ast::FunctionWithBody};
use cairo_lang_utils::Upcast;

#[expect(clippy::needless_pass_by_value)]
pub fn into_proc_macro_result(
    args: TokenStream,
    item: TokenStream,
    handler: impl Fn(
        &TokenStream,
        &TokenStream,
        &mut Vec<Diagnostic>,
    ) -> Result<TokenStream, Diagnostics>,
) -> ProcMacroResult {
    let mut warns = vec![]; // `Vec<Diagnostic>` instead of `Diagnostics` because `Diagnostics` does not allow to push ready `Diagnostic`

    match handler(&args, &item, &mut warns) {
        Ok(item) => ProcMacroResult::new(item).with_diagnostics(warns.into()),
        Err(mut diagnostics) => {
            diagnostics.extend(warns);
            ProcMacroResult::new(item).with_diagnostics(diagnostics)
        }
    }
}

pub fn with_parsed_values<Collector>(
    args: &TokenStream,
    item: &TokenStream,
    warns: &mut Vec<Diagnostic>,
    handler: impl Fn(
        //func item
        &SimpleParserDatabase,
        &FunctionWithBody,
        //args
        &SimpleParserDatabase,
        Arguments,
        //warns
        &mut Vec<Diagnostic>,
    ) -> Result<TokenStream, Diagnostics>,
) -> Result<TokenStream, Diagnostics>
where
    Collector: AttributeInfo,
{
    let (db, func) = parse::<Collector>(item)?;

    let db = db.upcast();

    // TODO: Restore assert_is_used_once::<Collector>(db, &func)?; in functions which require this check.

    let (args_db, args) = parse_args(args);
    let args_db = args_db.upcast();

    let args = Arguments::new::<Collector>(args_db, args, warns);

    handler(db, &func, args_db, args, warns)
}

pub fn no_fuzzer_attribute(db: &SimpleParserDatabase, func: &FunctionWithBody) -> bool {
    const FUZZER_ATTRIBUTES: [&str; 3] = [
        FuzzerCollector::ATTR_NAME,
        FuzzerWrapperCollector::ATTR_NAME,
        FuzzerConfigCollector::ATTR_NAME,
    ];

    func.attributes(db)
        .elements(db)
        .any(|attr| {
            FUZZER_ATTRIBUTES.contains(
                &attr
                    .attr(db)
                    .as_syntax_node()
                    .get_text_without_trivia(db)
                    .as_str(),
            )
        })
        .not()
}
