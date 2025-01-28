use crate::attributes::fuzzer::FUZZABLE_PATH;
use crate::attributes::internal_config_statement::InternalConfigStatementCollector;
use crate::attributes::test::TestCollector;
use crate::attributes::AttributeInfo;
use crate::utils::TypedSyntaxNodeAsText;
use crate::{
    args::Arguments,
    attributes::AttributeCollector,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::ast::Param;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::{
    ast::{Condition, Expr, FunctionWithBody, Statement},
    db::SyntaxGroup,
    helpers::GetIdentifier,
};
use indoc::formatdoc;

#[allow(clippy::needless_pass_by_value)]
pub fn extend_with_config_cheatcodes<Collector>(
    args: TokenStream,
    item: TokenStream,
) -> ProcMacroResult
where
    Collector: AttributeCollector,
{
    into_proc_macro_result(args, item, |args, item, warns| {
        with_parsed_values::<Collector>(args, item, warns, with_config_cheatcodes::<Collector>)
    })
}

fn with_config_cheatcodes<Collector>(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    args_db: &dyn SyntaxGroup,
    args: Arguments,
    warns: &mut Vec<Diagnostic>,
) -> Result<String, Diagnostics>
where
    Collector: AttributeCollector,
{
    let value = Collector::args_into_config_expression(args_db, args, warns)?;

    let cheatcode_name = Collector::CHEATCODE_NAME;

    let config_cheatcode = formatdoc!(
        r"
            let mut data = array![];

            {value}
            .serialize(ref data);

            starknet::testing::cheatcode::<'{cheatcode_name}'>(data.span());
        "
    );

    if Collector::ATTR_NAME == "fuzzer" {
        Ok(append_fuzzer_config_statements(db, func, &config_cheatcode))
    } else {
        Ok(append_config_statements(db, func, &config_cheatcode))
    }
}

pub fn append_config_statements(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    config_statements: &str,
) -> String {
    let vis = func.visibility(db).as_text(db);
    let attrs = func.attributes(db).as_text(db);
    let declaration = func.declaration(db).as_text(db);

    let (statements, if_content) = get_statements(db, func);

    formatdoc!(
        "
            {attrs}
            {vis} {declaration} {{
                if snforge_std::_internals::_is_config_run() {{
                    {if_content}

                    {config_statements}

                    return;
                }}

                {statements}
            }}
        "
    )
}

#[allow(clippy::too_many_lines)]
pub fn append_fuzzer_config_statements(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
    config_statements: &str,
) -> String {
    let attr_list = func.attributes(db);
    let test_or_executable_attrs =
        if let Some(test_attr) = attr_list.find_attr(db, TestCollector::ATTR_NAME) {
            vec![test_attr]
        } else {
            [
                attr_list.query_attr(db, "snforge_internal_test_executable"),
                attr_list.query_attr(db, InternalConfigStatementCollector::ATTR_NAME),
            ]
            .concat()
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
    let function_kw = func.declaration(db).function_kw(db).as_text(db);
    let name = func.declaration(db).name(db).as_text(db);

    let signature = func.declaration(db).signature(db).as_text(db);

    let fuzzer_assignments = extract_and_modify_params(
        db,
        func,
        |param| {
            format!(
                "let {}{} = {FUZZABLE_PATH}::generate();",
                param.name(db).as_text(db),
                param.type_clause(db).as_text(db),
            )
        },
        "\n",
    );

    let blank_values_for_config_run =
        extract_and_modify_params(db, func, |_param| format!("{FUZZABLE_PATH}::blank()"), ", ");

    let params_call = extract_and_modify_params(db, func, |param| param.name(db).as_text(db), ", ");

    let internal_config_attr = InternalConfigStatementCollector::ATTR_NAME;
    let actual_body_fn_name = format!("{name}_actual_body");

    let (statements, if_content) = get_statements(db, func);

    formatdoc!(
        "
            {test_or_executable_attrs}
            {vis} {function_kw} {name}() {{
                if snforge_std::_internals::_is_config_run() {{
                    {if_content}

                    {config_statements}

                    {actual_body_fn_name}({blank_values_for_config_run});

                    return;
                }}
                {fuzzer_assignments}
                {actual_body_fn_name}({params_call});
            }}

            {actual_body_fn_attrs}
            #[{internal_config_attr}]
            {function_kw} {actual_body_fn_name}{signature} {{
                {statements}
            }}
        "
    )
}

fn extract_and_modify_params<F>(
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

// Gets test statements and content of `if` statement that checks if function is run in config mode
fn get_statements(db: &dyn SyntaxGroup, func: &FunctionWithBody) -> (String, String) {
    let statements = func.body(db).statements(db).elements(db);

    let if_content = statements.first().and_then(|stmt| {
        // first statement is `if`
        let Statement::Expr(expr) = stmt else {
            return None;
        };
        let Expr::If(if_expr) = expr.expr(db) else {
            return None;
        };
        // its condition is function call
        let Condition::Expr(expr) = if_expr.condition(db) else {
            return None;
        };
        let Expr::FunctionCall(expr) = expr.expr(db) else {
            return None;
        };

        // this function is named "snforge_std::_internals::_is_config_run"
        let segments = expr.path(db).elements(db);

        let [snforge_std, cheatcode, is_config_run] = segments.as_slice() else {
            return None;
        };

        if snforge_std.identifier(db) != "snforge_std"
            || cheatcode.identifier(db) != "_internals"
            || is_config_run.identifier(db) != "_is_config_run"
        {
            return None;
        }

        let statements = if_expr.if_block(db).statements(db).elements(db);

        // omit last one (`return;`) as it have to be inserted after all new statements
        Some(
            statements[..statements.len() - 1]
                .iter()
                .fold(String::new(), |acc, statement| {
                    acc + "\n" + &statement.as_text(db)
                }),
        )
    });

    // there was already config check, omit it and collect remaining statements
    let statements = if if_content.is_some() {
        &statements[1..]
    } else {
        &statements[..]
    }
    .iter()
    .fold(String::new(), |acc, stmt| acc + &stmt.as_text(db));

    (statements, if_content.unwrap_or_default())
}
