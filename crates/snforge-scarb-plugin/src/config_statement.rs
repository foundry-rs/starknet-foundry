use crate::utils::{create_single_token, get_statements};
use crate::{
    args::Arguments,
    attributes::AttributeCollector,
    common::{into_proc_macro_result, with_parsed_values},
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::ast::FunctionWithBody;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;

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
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    args_db: &SimpleParserDatabase,
    args: Arguments,
    warns: &mut Vec<Diagnostic>,
) -> Result<TokenStream, Diagnostics>
where
    Collector: AttributeCollector,
{
    let value = Collector::args_into_config_expression(args_db, args, warns)?;

    let cheatcode_name = Collector::CHEATCODE_NAME;
    let cheatcode = create_single_token(format!("'{cheatcode_name}'"));
    let cheatcode = quote! {
        starknet::testing::cheatcode::<#cheatcode>(data.span());
    };

    let config_cheatcode = quote!(
            let mut data = array![];

            #value
            .serialize(ref data);

            #cheatcode
    );

    Ok(append_config_statements(db, func, config_cheatcode))
}

#[expect(clippy::needless_pass_by_value)]
pub fn append_config_statements(
    db: &SimpleParserDatabase,
    func: &FunctionWithBody,
    config_statements: TokenStream,
) -> TokenStream {
    let vis = func.visibility(db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, db);

    let attrs = func.attributes(db).as_syntax_node();
    let attrs = SyntaxNodeWithDb::new(&attrs, db);

    let declaration = func.declaration(db).as_syntax_node();
    let declaration = SyntaxNodeWithDb::new(&declaration, db);

    let (statements, if_content) = get_statements(db, func);

    quote!(
        #attrs
        #vis #declaration {
            if snforge_std::_internals::is_config_run() {
                #if_content

                #config_statements

                return;
            }

            #statements
        }
    )
}
