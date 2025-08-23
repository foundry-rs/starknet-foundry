use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use regex::Regex;

use crate::args::Arguments;
use crate::args::unnamed::UnnamedArgs;
use crate::attributes::test::TestCollector;
use crate::attributes::{AttributeInfo, ErrorExt};
use crate::common::{has_fuzzer_attribute, into_proc_macro_result, with_parsed_values};
use crate::utils::SyntaxNodeUtils;
use crate::{create_single_token, format_ident};

static RE_SANITIZE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"[^a-zA-Z0-9]+").expect("Failed to create regex"));

fn sanitize_str(raw: &str) -> String {
    let s = RE_SANITIZE
        .replace_all(raw, "_")
        .to_lowercase()
        .trim_matches('_')
        .to_string();
    if s.is_empty() { "_empty".into() } else { s }
}

fn expr_label(expr: &Expr, db: &SimpleParserDatabase) -> String {
    sanitize_str(&expr.as_syntax_node().get_text(db))
}

fn case_fn_name(
    func_name: &str,
    unnamed_args: &UnnamedArgs,
    args_db: &SimpleParserDatabase,
) -> String {
    let parts = unnamed_args
        .iter()
        .map(|(_, expr)| expr_label(expr, args_db))
        .collect::<Vec<_>>();

    let suffix = if parts.is_empty() {
        "case".to_string()
    } else {
        parts.join("_")
    };

    format!("{func_name}_{suffix}")
}

pub struct TestCaseCollector;

impl AttributeInfo for TestCaseCollector {
    const ATTR_NAME: &'static str = "test_case";
}

use std::collections::HashMap;
use std::sync::LazyLock;

#[must_use]
pub fn vec_pairs_to_map<T>(pairs: Vec<(usize, T)>) -> HashMap<usize, T> {
    pairs.into_iter().collect()
}

#[must_use]
pub fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, test_case_internal)
}

pub fn get_test_case_name(
    func_name: &str,
    arguments: &Arguments,
    db: &SimpleParserDatabase,
) -> Result<Option<String>, Diagnostics> {
    let named_args = arguments.named();
    let test_case_name = named_args
        .as_once_optional("name")?
        .map(|arg| {
            let expr = arg;
            match expr {
                Expr::String(_) | Expr::ShortString(_) => {
                    Ok(expr.as_syntax_node().get_text(db).to_string())
                }
                _ => Err(TestCaseCollector::error(
                    "The 'name' argument must be a string literal.",
                )),
            }
        })
        .transpose()?;

    if let Some(ref name) = test_case_name {
        let sanitized = sanitize_str(name);
        let test_case_name = format!("{func_name}_{sanitized}");
        return Ok(Some(test_case_name));
    }
    Ok(None)
}

fn test_case_internal(
    args: &TokenStream,
    item: &TokenStream,
    warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    with_parsed_values::<TestCaseCollector>(
        args,
        item,
        warns,
        |func_db, func, args_db, arguments, _warns| {
            let unnamed_args = arguments.unnamed();
            let param_count = func
                .declaration(func_db)
                .signature(func_db)
                .parameters(func_db)
                .elements(func_db)
                .len();

            if param_count == 0 {
                return Err(Diagnostics::from(TestCaseCollector::error(
                    "The function must have at least one parameter to use",
                )));
            }

            if param_count != unnamed_args.len() {
                return Err(Diagnostics::from(TestCaseCollector::error(format!(
                    "Expected {} parameters, but got {}",
                    param_count,
                    unnamed_args.len()
                ))));
            }

            let func_name = func.declaration(func_db).name(func_db).text(func_db);
            let test_case_name = get_test_case_name(&func_name, &arguments, args_db)?;
            let case_fn_name = test_case_name.unwrap_or(case_fn_name(
                func_name.as_ref(),
                &UnnamedArgs::new(&vec_pairs_to_map(
                    unnamed_args
                        .clone()
                        .into_iter()
                        .map(|(i, expr)| (i, expr.clone()))
                        .collect::<Vec<_>>(),
                )),
                args_db,
            ));

            let attr_list = func.attributes(func_db);
            let has_fuzzer = has_fuzzer_attribute(func_db, func);

            // We do not want to copy the `#[test]` attribute if there is no `#[fuzzer]` attribute.
            let filtered_fn_attrs = attr_list
                .elements(func_db)
                .filter(|attr| {
                    let test_attr_text = format!("#[{}]", TestCollector::ATTR_NAME);
                    let attr_text = attr.as_syntax_node().get_text(func_db);
                    let attr_text = attr_text.trim();
                    let is_test_attr = attr_text == test_attr_text;

                    !is_test_attr || has_fuzzer
                })
                .map(|attr| attr.to_token_stream(func_db))
                .fold(TokenStream::empty(), |mut acc, token| {
                    acc.extend(token);
                    acc
                });

            let signature = func
                .declaration(func_db)
                .signature(func_db)
                .as_syntax_node();
            let signature = SyntaxNodeWithDb::new(&signature, func_db);

            let func_body_node = func.body(func_db).as_syntax_node();
            let func_body = SyntaxNodeWithDb::new(&func_body_node, func_db);

            let func_name = format_ident!("{}", func_name);
            let func_node = quote!(
                #filtered_fn_attrs
                fn #func_name #signature
                #func_body
            );

            let call_args = unnamed_args.clone().into_iter();

            let call_args = call_args
                .into_iter()
                .map(|(_, expr)| expr.as_syntax_node().get_text(args_db))
                .collect::<Vec<_>>()
                .join(", ")
                .to_string();

            let call_args = format_ident!("({})", call_args);

            let case_fn_name = format_ident!("{}", case_fn_name);

            let out_of_gas = create_single_token("'Out of gas'");

            Ok(quote!(
                #[implicit_precedence(core::pedersen::Pedersen, core::RangeCheck, core::integer::Bitwise, core::ec::EcOp, core::poseidon::Poseidon, core::SegmentArena, core::circuit::RangeCheck96, core::circuit::AddMod, core::circuit::MulMod, core::gas::GasBuiltin, System)]
                #[snforge_internal_test_executable]
                fn #case_fn_name(mut _data: Span<felt252>) -> Span::<felt252> {
                    core::internal::require_implicit::<System>();
                    core::internal::revoke_ap_tracking();
                    core::option::OptionTraitImpl::expect(core::gas::withdraw_gas(), #out_of_gas);

                    core::option::OptionTraitImpl::expect(
                        core::gas::withdraw_gas_all(core::gas::get_builtin_costs()), #out_of_gas
                    );
                    #func_name #call_args;

                    let mut arr = ArrayTrait::new();
                    core::array::ArrayTrait::span(@arr)
                }

                #func_node
            ))
        },
    )
}
