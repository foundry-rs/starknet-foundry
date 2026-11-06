use crate::args::Arguments;
use crate::args::unnamed::UnnamedArgs;
use crate::attributes::finalize::{FinalizeCollector, config_fn_ident};
use crate::attributes::test::{TestCollector, test_func_with_attrs};
use crate::attributes::test_case::name::test_case_name;
use crate::attributes::{AttributeInfo, ErrorExt};
use crate::common::{
    has_fuzzer_attribute, has_test_attribute, into_proc_macro_result, with_parsed_values,
};
use crate::utils::SyntaxNodeUtils;
use crate::{create_single_token, format_ident};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::FunctionWithBody;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};

mod name;

pub struct TestCaseCollector;

impl AttributeInfo for TestCaseCollector {
    const ATTR_NAME: &'static str = "test_case";
}

#[must_use]
pub fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<TestCaseCollector>(args, item, warns, test_case_internal)
    })
}

#[expect(clippy::needless_pass_by_value)]
fn test_case_internal(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    args_db: &SimpleParserDatabase,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics> {
    let named_args = args.named();
    named_args.allow_only::<TestCaseCollector>(&["name"])?;

    let unnamed_args = args.unnamed();
    ensure_params_valid(func, &args.unnamed(), db)?;

    let func_name = func.declaration(db).name(db);

    let name_arg = named_args.as_once_optional("name")?;
    let case_fn_name = test_case_name(&func_name.text(db), name_arg, &unnamed_args, args_db)?;
    let filtered_fn_attrs = collect_preserved_attributes_for_test_case(func, db);

    let signature = func.declaration(db).signature(db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, db);

    let func_body = func.body(db).as_syntax_node();
    let func_body = SyntaxNodeWithDb::new(&func_body, db);

    let case_fn_name = format_ident!("{}", case_fn_name);
    let case_fn_name = TokenStream::new(vec![case_fn_name]);

    let config_fn = config_fn_ident(&func_name.text(db));

    let func_name = func_name.to_token_stream(db);

    let call_args = args_to_token_stream(&unnamed_args, args_db);

    let test_func_with_attrs =
        test_func_with_attrs(&case_fn_name, &func_name, &call_args, &config_fn);

    // Both `#[fuzzer]` and `#[test]` still being here means neither has expanded yet. `#[test]`
    // attaches the finalize marker itself once it does, so attaching one here would make two.
    // Note that `collect_preserved_attributes_for_test_case` drops any marker already on the
    // function - that is safe, because a marker can only be there once `#[fuzzer]` has expanded,
    // and then this condition is false and we re-attach it below.
    let skip_finalize = has_fuzzer_attribute(db, func) && has_test_attribute(db, func);
    let finalize_attr = if skip_finalize {
        TokenStream::empty()
    } else {
        let finalize = create_single_token(FinalizeCollector::ATTR_NAME);
        quote!(#[#finalize])
    };

    let func_ident = quote!(
        #filtered_fn_attrs

        #finalize_attr
        fn #func_name #signature
        #func_body
    );

    Ok(quote!(
        #test_func_with_attrs

        #func_ident
    ))
}

fn args_to_token_stream(args: &UnnamedArgs, db: &SimpleParserDatabase) -> TokenStream {
    args.iter()
        .map(|(_, expr)| {
            let expr = expr.as_syntax_node();
            let expr = SyntaxNodeWithDb::new(&expr, db);
            quote! { #expr, }
        })
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        })
}

fn ensure_params_valid(
    func: &FunctionWithBody,
    unnamed_args: &UnnamedArgs,
    db: &SimpleParserDatabase,
) -> Result<(), Diagnostics> {
    let param_count = func
        .declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .len();

    if param_count == 0 {
        return Err(Diagnostics::from(TestCaseCollector::error(
            "The function must have at least one parameter to use #[test_case] attribute",
        )));
    }

    if param_count != unnamed_args.len() {
        return Err(Diagnostics::from(TestCaseCollector::error(format!(
            "Expected {} arguments, but got {}",
            param_count,
            unnamed_args.len()
        ))));
    }

    Ok(())
}

fn collect_preserved_attributes_for_test_case(
    func: &FunctionWithBody,
    func_db: &SimpleParserDatabase,
) -> TokenStream {
    let attr_list = func.attributes(func_db);
    let has_fuzzer = has_fuzzer_attribute(func_db, func);

    let test_attrs = attr_list
        .query_attr(func_db, TestCollector::ATTR_NAME)
        .collect::<Vec<_>>();
    let finalize_attrs = attr_list
        .query_attr(func_db, FinalizeCollector::ATTR_NAME)
        .collect::<Vec<_>>();

    // We do not want to copy the `#[test]` attribute unless the function has `#[fuzzer]`.
    // We also do not want to copy `#[__snforge_finalize]` because it will be added later.
    attr_list
        .elements(func_db)
        .filter(|attr| {
            let is_test_attr = test_attrs.contains(attr);
            let is_finalize_attr = finalize_attrs.contains(attr);

            (!is_test_attr || has_fuzzer) && !is_finalize_attr
        })
        .map(|attr| attr.to_token_stream(func_db))
        .fold(TokenStream::empty(), |mut acc, token| {
            acc.extend(token);
            acc
        })
}
