use super::{AttributeCollector, AttributeInfo, AttributeTypeData, ErrorExt};
use crate::args::Arguments;
use crate::asserts::assert_is_used_once;
use crate::attributes::fuzzer::wrapper::FuzzerWrapperCollector;
use crate::cairo_expression::CairoExpression;
use crate::common::into_proc_macro_result;
use crate::config_statement::extend_with_config_cheatcodes;
use crate::parse::parse;
use crate::types::{Number, ParseFromExpr};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_utils::Upcast;
use indoc::formatdoc;
use num_bigint::BigInt;

pub mod wrapper;

pub struct FuzzerConfigCollector;

impl AttributeInfo for FuzzerConfigCollector {
    const ATTR_NAME: &'static str = "__fuzzer_config";
}

pub struct FuzzerCollector;

impl AttributeInfo for FuzzerCollector {
    const ATTR_NAME: &'static str = "fuzzer";
}

impl AttributeTypeData for FuzzerCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fuzzer";
}

impl AttributeCollector for FuzzerCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let seed = named_args
            .as_once_optional("seed")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "seed"))
            .transpose()?;

        let runs = named_args
            .as_once_optional("runs")?
            .map(|arg| Number::parse_from_expr::<Self>(db, arg, "runs"))
            .transpose()?;

        if let Some(Number(ref runs)) = runs {
            if runs <= &BigInt::from(0) {
                Err(Self::error("runs must be greater than 0"))?;
            }
        }

        let seed = seed.as_cairo_expression();
        let runs = runs.as_cairo_expression();

        Ok(format!(
            "snforge_std::_internals::_config_types::FuzzerConfig {{ seed: {seed}, runs: {runs} }}"
        ))
    }
}

#[must_use]
pub fn fuzzer(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, fuzzer_internal)
}

#[must_use]
pub fn fuzzer_config(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<FuzzerCollector>(args, item)
}

#[expect(clippy::ptr_arg)]
fn fuzzer_internal(
    args: &TokenStream,
    item: &TokenStream,
    _warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics> {
    let item = item.to_string();
    let (db, func) = parse::<FuzzerCollector>(&item)?;
    let db = db.upcast();

    assert_is_used_once::<FuzzerCollector>(db, &func)?;

    let attrs = func.attributes(db).as_syntax_node().get_text(db);
    let body = func.body(db).as_syntax_node().get_text(db);
    let declaration = func.declaration(db).as_syntax_node().get_text(db);

    Ok(formatdoc!(
        "
            {attrs}
            #[{}{}]
            #[{}]
            {declaration} {body}
        ",
        FuzzerConfigCollector::ATTR_NAME,
        args.to_string(),
        FuzzerWrapperCollector::ATTR_NAME
    ))
}
