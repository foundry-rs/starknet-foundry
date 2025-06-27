use crate::args::Arguments;
use crate::attributes::internal_config_statement::InternalConfigStatementCollector;
use crate::attributes::test::TestCollector;
use crate::attributes::AttributeInfo;
use crate::common::{into_proc_macro_result, with_parsed_values};
use crate::utils::{get_statements, TypedSyntaxNodeAsText};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::ast::{FunctionWithBody, Param};
use cairo_lang_syntax::node::db::SyntaxGroup;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use indoc::formatdoc;

pub struct FuzzerWrapperCollector;

impl AttributeInfo for FuzzerWrapperCollector {
    const ATTR_NAME: &'static str = "__fuzzer_wrapper";
}

#[must_use]
pub fn fuzzer_wrapper(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<FuzzerWrapperCollector>(args, item, warns, fuzzer_wrapper_internal)
    })
}

#[expect(clippy::ptr_arg)]
#[expect(clippy::needless_pass_by_value)]
fn fuzzer_wrapper_internal(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    _args_db: &dyn SyntaxGroup,
    args: Arguments,
    _warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics> {
    args.assert_is_empty::<FuzzerWrapperCollector>()?;

    let attr_list = func.attributes(db);
    let test_or_executable_attrs =
        if let Some(test_attr) = attr_list.find_attr(db, TestCollector::ATTR_NAME) {
            vec![test_attr]
        } else {
            attr_list.query_attr(db, InternalConfigStatementCollector::ATTR_NAME)
        };

    let actual_body_fn_attrs = attr_list
        .elements(db)
        .into_iter()
        .filter(|attr| !test_or_executable_attrs.contains(attr))
        .map(|attr| attr.as_text(db))
        .collect::<Vec<String>>()
        .join("\n");

    let test_or_executable_attrs = test_or_executable_attrs
        .iter()
        .map(|attr| attr.as_text(db))
        .collect::<Vec<String>>()
        .join("\n");

    let vis = func.visibility(db).as_text(db);
    let name = func.declaration(db).name(db).as_text(db);

    let signature = func.declaration(db).signature(db).as_text(db);

    let fuzzer_assignments = extract_and_transform_params(
        db,
        func,
        |param| {
            format!(
                r"
                let {}{} = snforge_std_compatibility::fuzzable::Fuzzable::generate();
                snforge_std_compatibility::_internals::save_fuzzer_arg(@{});
                ",
                param.name(db).as_text(db),
                param.type_clause(db).as_text(db),
                param.name(db).as_text(db),
            )
        },
        "\n",
    );

    let blank_values_for_config_run = extract_and_transform_params(
        db,
        func,
        |_param| "snforge_std_compatibility::fuzzable::Fuzzable::blank()".to_string(),
        ", ",
    );

    let arguments_list =
        extract_and_transform_params(db, func, |param| param.name(db).as_text(db), ", ");

    let internal_config_attr = InternalConfigStatementCollector::ATTR_NAME;
    let actual_body_fn_name = format!("{name}_actual_body");

    let (statements, if_content) = get_statements(db, func);

    Ok(formatdoc!(
        "
            {test_or_executable_attrs}
            {vis} fn {name}() {{
                if snforge_std_compatibility::_internals::is_config_run() {{
                    {if_content}

                    {actual_body_fn_name}({blank_values_for_config_run});

                    return;
                }}
                {fuzzer_assignments}
                {actual_body_fn_name}({arguments_list});
            }}

            {actual_body_fn_attrs}
            #[{internal_config_attr}]
            fn {actual_body_fn_name}{signature} {{
                {statements}
            }}
        "
    ))
}

fn extract_and_transform_params<F>(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    transformer: F,
    separator: &str,
) -> String
where
    F: Fn(&Param) -> String,
{
    func.declaration(db)
        .signature(db)
        .parameters(db)
        .elements(db)
        .iter()
        .map(transformer)
        .collect::<Vec<String>>()
        .join(separator)
}
