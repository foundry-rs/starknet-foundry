use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{Expr, OptionStructArgExpr, StructArg};
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

use crate::args::unnamed::UnnamedArgs;
use crate::attributes::{AttributeInfo, ErrorExt};
use crate::common::{has_fuzzer_attribute, into_proc_macro_result, with_parsed_values};
use crate::utils::SyntaxNodeUtils;
use crate::{create_single_token, format_ident};

fn hash8(s: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", h)
}

fn sanitize_ident_fragment(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut prev_us = false;
    for ch in raw.chars() {
        let m = if ch.is_ascii_alphanumeric() || ch == '_' {
            ch
        } else {
            '_'
        };
        if m == '_' {
            if !prev_us {
                out.push('_');
                prev_us = true;
            }
        } else {
            out.push(m);
            prev_us = false;
        }
    }
    let s = out.trim_matches('_').to_lowercase();
    if s.is_empty() { "x".to_string() } else { s }
}

fn shorten_with_hash(s: &str, max_len: usize, orig_for_hash: &str) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    format!("{}_{}", &s[..max_len], &hash8(orig_for_hash)[..6])
}

fn struct_arg_label(struct_arg: StructArg, db: &SimpleParserDatabase, budget: usize) -> String {
    match struct_arg {
        StructArg::StructArgSingle(single) => {
            let arg_expr = single.arg_expr(db);
            let label = match arg_expr {
                OptionStructArgExpr::Empty(_) => "empty".to_string(),
                OptionStructArgExpr::StructArgExpr(struct_arg_expr) => {
                    let expr = struct_arg_expr.expr(db);
                    expr_label(&expr, db, budget)
                }
            };
            label
        }
        StructArg::StructArgTail(tail) => {
            let label = tail.as_syntax_node().get_text(db);
            sanitize_ident_fragment(&label)
        }
    }
}

fn expr_label(expr: &Expr, db: &SimpleParserDatabase, budget: usize) -> String {
    let raw = expr.as_syntax_node().get_text(db);

    let base = match expr {
        Expr::Path(p) => {
            sanitize_ident_fragment(&p.as_syntax_node().get_text(db).replace("::", "_"))
        }
        Expr::Literal(n) => sanitize_ident_fragment(&n.as_syntax_node().get_text(db)),
        Expr::ShortString(s) => {
            let t = s
                .as_syntax_node()
                .get_text(db)
                .trim_matches('\'')
                .to_string();
            sanitize_ident_fragment(&t)
        }
        Expr::String(s) => {
            let t = s
                .as_syntax_node()
                .get_text(db)
                .trim_matches('"')
                .to_string();
            sanitize_ident_fragment(&t)
        }
        Expr::True(_) => "true".into(),
        Expr::False(_) => "false".into(),

        Expr::Unary(u) => {
            let op = u.op(db).as_syntax_node().get_text(db);
            let inner = expr_label(&u.expr(db), db, budget.saturating_sub(4));
            match op.as_str() {
                "-" => format!("neg{}", inner),
                "!" => format!("not_{}", inner),
                _ => sanitize_ident_fragment(&format!("op{}_{}", op, inner)),
            }
        }

        Expr::Tuple(t) => {
            let parts = t
                .expressions(db)
                .elements(db)
                .map(|e| expr_label(&e, db, budget / 3.max(1))) // prosta heurystyka budżetu
                .collect::<Vec<_>>();
            parts.join("_")
        }

        Expr::FunctionCall(_call) => {
            // TODO
            "function_call".to_string()
        }

        Expr::StructCtorCall(sc) => {
            let ty = sanitize_ident_fragment(
                &sc.path(db).as_syntax_node().get_text(db).replace("::", "_"),
            );
            let fields = sc
                .arguments(db)
                .arguments(db)
                .elements(db)
                .map(|f| struct_arg_label(f, db, budget))
                .collect::<Vec<_>>()
                .join("_");
            if fields.is_empty() {
                ty
            } else {
                format!("{}_{}", ty, fields)
            }
        }

        Expr::Indexed(_ix) => {
            // TODO
            "indexed".to_string()
        }

        _ => format!("expr_{}", &hash8(&raw)[..6]),
    };

    shorten_with_hash(&base, budget, &raw)
}

fn case_fn_name(
    func_name: String,
    unnamed_args: UnnamedArgs,
    args_db: &SimpleParserDatabase,
) -> String {
    let parts = unnamed_args
        .iter()
        .map(|(_, expr)| expr_label(expr, args_db, 32))
        .collect::<Vec<_>>();

    let suffix = if parts.is_empty() {
        "case".to_string()
    } else {
        parts.join("_")
    };

    format!("{}_{}", func_name, suffix)
}

// fn case_fn_name(
//     func: FunctionWithBody,
//     args: Arguments,
//     args_db: &SimpleParserDatabase,
//     func_db: &SimpleParserDatabase,
// ) -> String {
//     let func_name = func
//         .declaration(func_db)
//         .name(func_db)
//         .as_syntax_node()
//         .get_text(func_db);

//     let sanitized_args = args
//         .unnamed_only::<TestCaseCollector>()
//         .expect("xyz")
//         .iter()
//         .map(|arg| arg.1.as_syntax_node().get_text(args_db))
//         .collect::<Vec<_>>()
//         .join("_");

//     format!("{}_{}", func_name, sanitized_args)
// }

pub struct TestCaseCollector;

impl AttributeInfo for TestCaseCollector {
    const ATTR_NAME: &'static str = "test_case";
}

use std::collections::HashMap;

pub fn vec_pairs_to_map<T>(pairs: Vec<(usize, T)>) -> HashMap<usize, T> {
    pairs.into_iter().collect()
}

fn to_test_case_name(func_name: String, expr: &Expr, args_db: &SimpleParserDatabase) -> String {
    let test_case_name = match expr {
        Expr::String(s) => s.as_syntax_node().get_text(args_db).to_string(),
        Expr::ShortString(s) => s.as_syntax_node().get_text(args_db).to_string(),
        _ => {
            panic!("Expected a string literal, found: {:?}", expr);
        }
    };

    let test_case_name = if test_case_name.is_empty() {
        "empty".to_string()
    } else {
        test_case_name
    };

    let sanitized = sanitize_ident_fragment(&test_case_name);
    format!("{}_{}", func_name, sanitized)
}

#[must_use]
pub fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, test_case_internal)
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
            let param_count = func
                .declaration(func_db)
                .signature(func_db)
                .parameters(func_db)
                .elements(func_db)
                .len();

            let unnamed_args = arguments.unnamed_only::<TestCaseCollector>()?;
            let is_name_passed = param_count + 1 == unnamed_args.len();

            // Number of args can be equal to n or n + 1, where n is
            // the number of function parameters. n + 1 is when the last arg
            // is the test case name.
            if param_count != unnamed_args.len() && param_count != unnamed_args.len() - 1 {
                let args_got = if is_name_passed {
                    unnamed_args.len() - 1
                } else {
                    unnamed_args.len()
                };
                return Err(Diagnostics::from(TestCaseCollector::error(format!(
                    "Expected {} parameters, but got {}",
                    param_count, args_got
                ))));
            }

            let func_name = func.declaration(func_db).name(func_db).text(func_db);
            let func_name_ident = format_ident!("{}", func_name);
            let has_fuzzer = has_fuzzer_attribute(func_db, func);

            let (case_fn_name, call_args) = if is_name_passed {
                let mut fn_args = unnamed_args.clone().into_iter().into_iter();
                let fn_arg_with_test_case_name =
                    fn_args.next().expect("Failed to get first arument");
                let fn_arg_with_test_case_name_expr = fn_arg_with_test_case_name.1;
                let case_fn_name = to_test_case_name(
                    func_name.to_string(),
                    &fn_arg_with_test_case_name_expr,
                    args_db,
                );
                (case_fn_name, fn_args)
            } else {
                let case_fn_name = case_fn_name(
                    func_name.to_string(),
                    UnnamedArgs::new(&vec_pairs_to_map(
                        unnamed_args
                            .clone()
                            .into_iter()
                            .map(|(i, expr)| (i, expr.clone()))
                            .collect::<Vec<_>>(),
                    )),
                    args_db,
                );
                (case_fn_name, unnamed_args.clone().into_iter())
            };

            let attr_list = func.attributes(func_db);

            let filtered_fn_attrs = attr_list
                .elements(func_db)
                .filter(|attr| {
                    let text = attr.as_syntax_node().get_text(func_db);
                    if text.contains("#[test]") && !has_fuzzer {
                        return false;
                    }
                    true
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

            let func_node = quote!(
                #filtered_fn_attrs
                fn #func_name_ident #signature
                #func_body
            );

            // let other_fn_attrs = attr_list
            //     .elements(func_db)
            //     .filter(|attr| {
            //         ![
            //             TestCaseCollector::ATTR_NAME,
            //             TestCollector::ATTR_NAME,
            //             FuzzerCollector::ATTR_NAME,
            //             FuzzerConfigCollector::ATTR_NAME,
            //             FuzzerWrapperCollector::ATTR_NAME,
            //         ]
            //         .contains(
            //             &attr
            //                 .attr(func_db)
            //                 .as_syntax_node()
            //                 .get_text(func_db)
            //                 .as_str(),
            //         )
            //     })
            //     .map(|attr| attr.to_token_stream(func_db))
            //     .fold(TokenStream::empty(), |mut acc, token| {
            //         acc.extend(token);
            //         acc
            //     });

            let call_args = call_args
                .into_iter()
                .map(|(_, expr)| expr.as_syntax_node().get_text(args_db))
                .collect::<Vec<_>>()
                .join(", ")
                .to_string();

            let call_args = format_ident!("({})", call_args);

            let case_fn_name = case_fn_name.replace("return_wrapper_actual_body_", "");
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
                    #func_name_ident #call_args;

                    let mut arr = ArrayTrait::new();
                    core::array::ArrayTrait::span(@arr)
                }

                #func_node
            ))
        },
    )
}
