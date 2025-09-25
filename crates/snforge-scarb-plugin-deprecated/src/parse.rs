use crate::attributes::{AttributeInfo, ErrorExt};
use cairo_lang_macro::Diagnostic;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::SyntaxFile;
use cairo_lang_syntax::node::{
    ast::{FunctionWithBody, ModuleItem, OptionArgListParenthesized},
    db::SyntaxGroup,
    helpers::QueryAttrs,
    TypedSyntaxNode,
};
use cairo_lang_utils::Upcast;
use indoc::formatdoc;

pub fn parse<T: AttributeInfo>(
    code: &str,
) -> Result<(SimpleParserDatabase, FunctionWithBody), Diagnostic> {
    let simple_db = SimpleParserDatabase::default();
    let (parsed_node, _diagnostics) = simple_db.parse_virtual_with_diagnostics(code);

    let db: &dyn SyntaxGroup = simple_db.upcast();
    let elements = SyntaxFile::from_syntax_node(db, parsed_node)
        .items(db)
        .elements(db);

    elements
        .into_iter()
        .find_map(|element| {
            if let ModuleItem::FreeFunction(func) = element {
                Some(func)
            } else {
                None
            }
        })
        .map(|func| (simple_db, func))
        .ok_or_else(|| T::error("can be used only on a function"))
}

struct InternalCollector;

impl AttributeInfo for InternalCollector {
    const ATTR_NAME: &'static str = "__SNFORGE_INTERNAL_ATTR__";
}

pub fn parse_args(args: &str) -> (SimpleParserDatabase, OptionArgListParenthesized) {
    let (simple_db, func) = parse::<InternalCollector>(&formatdoc!(
        "
            #[{}{args}]
            fn __SNFORGE_INTERNAL_FN__(){{}}
        ",
        InternalCollector::ATTR_NAME
    ))
    .expect("Parsing the arguments shouldn't fail at this stage"); // Arguments were parsed previously, so they should pass parsing here

    let db = simple_db.upcast();

    let args = func
        .attributes(db)
        .find_attr(db, InternalCollector::ATTR_NAME)
        .unwrap()
        .arguments(db);

    (simple_db, args)
}
