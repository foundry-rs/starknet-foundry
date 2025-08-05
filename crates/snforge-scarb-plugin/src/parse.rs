use crate::attributes::{AttributeInfo, ErrorExt};
use crate::utils::create_single_token;
use cairo_lang_macro::{Diagnostic, TokenStream, TokenTree, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::SyntaxFile;
use cairo_lang_syntax::node::{
    TypedSyntaxNode,
    ast::{FunctionWithBody, ModuleItem, OptionArgListParenthesized},
    helpers::QueryAttrs,
};
use cairo_lang_utils::Upcast;

pub fn parse<T: AttributeInfo>(
    code: &TokenStream,
) -> Result<(SimpleParserDatabase, FunctionWithBody), Diagnostic> {
    let simple_db = SimpleParserDatabase::default();
    let (parsed_node, diagnostics) = simple_db.parse_token_stream(code);

    if !diagnostics.is_empty() {
        return Err(Diagnostic::error("Failed because of invalid syntax"));
    }

    let db = simple_db.upcast();
    let function = SyntaxFile::from_syntax_node(db, parsed_node)
        .items(db)
        .elements(db)
        .find_map(|element| {
            if let ModuleItem::FreeFunction(func) = element {
                Some(func)
            } else {
                None
            }
        });

    match function {
        Some(func) => Ok((simple_db, func)),
        None => Err(T::error("can be used only on a function")),
    }
}

struct InternalCollector;

impl AttributeInfo for InternalCollector {
    const ATTR_NAME: &'static str = "__SNFORGE_INTERNAL_ATTR__";
}

pub fn parse_args(args: &TokenStream) -> (SimpleParserDatabase, OptionArgListParenthesized) {
    let attr_name = create_single_token(InternalCollector::ATTR_NAME);
    let args = args.clone();
    let mut token_stream = quote! {
        #[#attr_name #args]
        fn __SNFORGE_INTERNAL_FN__(){{}}
    };
    if !token_stream.is_empty() {
        match &mut token_stream.tokens[0] {
            TokenTree::Ident(ident) => {
                ident.span.start = 0;
            }
        }
    }
    let (simple_db, func) = parse::<InternalCollector>(&token_stream)
        .expect("Parsing the arguments shouldn't fail at this stage"); // Arguments were parsed previously, so they should pass parsing here

    let db = simple_db.upcast();

    let args = func
        .attributes(db)
        .find_attr(db, InternalCollector::ATTR_NAME)
        .unwrap()
        .arguments(db);

    (simple_db, args)
}
