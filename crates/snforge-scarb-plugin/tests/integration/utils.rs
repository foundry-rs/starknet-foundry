use cairo_lang_formatter::{CairoFormatter, FormatterConfig};
use cairo_lang_macro::{Diagnostic, ProcMacroResult, TokenStream, TokenTree, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{ModuleItem, SyntaxFile};
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_syntax::node::{Terminal, TypedSyntaxNode};
use std::collections::HashSet;

pub fn empty_function() -> TokenStream {
    quote!(
        fn empty_fn() {}
    )
}

pub fn assert_diagnostics(result: &ProcMacroResult, expected: &[Diagnostic]) {
    let diagnostics: HashSet<_> = result.diagnostics.iter().collect();
    let expected: HashSet<_> = expected.iter().collect();

    let remaining_diagnostics: Vec<_> = diagnostics
        .difference(&expected)
        .map(|diff| {
            format!(
                "Diagnostic where emitted and unexpected:\n {:?} => \"{}\"\n",
                diff.severity(),
                diff.message(),
            )
        })
        .collect();

    let not_emitted_diagnostics: Vec<_> = expected
        .difference(&diagnostics)
        .map(|diff| {
            format!(
                "Diagnostic where expected but not emitted:\n {:?} => \"{}\"\n",
                diff.severity(),
                diff.message(),
            )
        })
        .collect();

    assert!(
        remaining_diagnostics.is_empty() && not_emitted_diagnostics.is_empty(),
        "\n---------------------\n\n{}\n---------------------",
        remaining_diagnostics.join("\n") + "\n" + &not_emitted_diagnostics.join("\n")
    );
}

pub fn format_output(result: &ProcMacroResult) -> String {
    let fmt = CairoFormatter::new(FormatterConfig::default());
    let code = result.token_stream.to_string();
    fmt.format_to_string(&code)
        .unwrap_or_else(|_| panic!("Failed to format provided code: {code}"))
        .into_output_text()
}

pub fn get_function(
    token_stream: &TokenStream,
    function_name: &str,
    skip_args: bool,
) -> TokenStream {
    let db = SimpleParserDatabase::default();
    let (parsed_node, _diagnostics) = db.parse_token_stream(token_stream);
    let syntax_file = SyntaxFile::from_syntax_node(&db, parsed_node);
    let function = syntax_file
        .items(&db)
        .elements(&db)
        .find_map(|e| {
            if let ModuleItem::FreeFunction(free_function) = e {
                if free_function.declaration(&db).name(&db).text(&db) == function_name {
                    Some(free_function.clone())
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();

    let vis = function.visibility(&db).as_syntax_node();
    let vis = SyntaxNodeWithDb::new(&vis, &db);

    let signature = function.declaration(&db).as_syntax_node();
    let signature = SyntaxNodeWithDb::new(&signature, &db);

    let body = function.body(&db).as_syntax_node();
    let body = SyntaxNodeWithDb::new(&body, &db);

    let attrs = function.attributes(&db).as_syntax_node();
    let attrs = SyntaxNodeWithDb::new(&attrs, &db);

    let mut token_stream = if skip_args {
        quote! {
            #vis #signature
            #body
        }
    } else {
        quote! {
            #attrs
            #vis #signature
            #body
        }
    };

    match &mut token_stream.tokens[0] {
        TokenTree::Ident(ident) => {
            ident.span.start = 0;
        }
    }

    token_stream
}
