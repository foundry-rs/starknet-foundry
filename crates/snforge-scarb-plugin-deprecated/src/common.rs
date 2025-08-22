use crate::{
    args::Arguments,
    asserts::assert_is_used_once,
    attributes::AttributeInfo,
    parse::{parse, parse_args},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup};
use cairo_lang_utils::Upcast;

#[expect(clippy::needless_pass_by_value)]
pub fn into_proc_macro_result(
    args: TokenStream,
    item: TokenStream,
    handler: impl Fn(&TokenStream, &TokenStream, &mut Vec<Diagnostic>) -> Result<String, Diagnostics>,
) -> ProcMacroResult {
    let mut warns = vec![]; // `Vec<Diagnostic>` instead of `Diagnostics` because `Diagnostics` does not allow to push ready `Diagnostic`

    match handler(&args, &item, &mut warns) {
        Ok(item) => ProcMacroResult::new(TokenStream::new(item)).with_diagnostics(warns.into()),
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
        &dyn SyntaxGroup,
        &FunctionWithBody,
        //args
        &dyn SyntaxGroup,
        Arguments,
        //warns
        &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics>,
) -> Result<String, Diagnostics>
where
    Collector: AttributeInfo,
{
    let item = item.to_string();
    let (db, func) = parse::<Collector>(&item)?;

    let db = db.upcast();

    let (args_db, args) = parse_args(&args.to_string());
    let args_db = args_db.upcast();

    let args = Arguments::new::<Collector>(args_db, args, warns);

    handler(db, &func, args_db, args, warns)
}
