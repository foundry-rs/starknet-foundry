use crate::attributes::{AttributeInfo, ErrorExt};
use cairo_lang_diagnostics::DiagnosticsBuilder;
use cairo_lang_filesystem::ids::{FileKind, FileLongId, VirtualFile};
use cairo_lang_macro::Diagnostic;
use cairo_lang_parser::{parser::Parser, utils::SimpleParserDatabase};
use cairo_lang_syntax::node::{
    ast::{FunctionWithBody, ModuleItem, OptionArgListParenthesized},
    db::SyntaxGroup,
    helpers::QueryAttrs,
};
use cairo_lang_utils::Upcast;
use indoc::formatdoc;
use std::sync::Arc;

pub fn parse<T: AttributeInfo>(
    code: &str,
) -> Result<(SimpleParserDatabase, FunctionWithBody), Diagnostic> {
    let simple_db = SimpleParserDatabase::default();
    let code = Arc::new(code.to_string());
    let db: &dyn SyntaxGroup = simple_db.upcast();
    // TODO(#2357): Use `db.parse_virtual` here instead of creating the virtual file manually
    let virtual_file = db.intern_file(FileLongId::Virtual(VirtualFile {
        parent: None,
        name: "test_function".into(),
        content: Arc::from(code.as_str()),
        code_mappings: Default::default(),
        kind: FileKind::Module,
    }));
    let mut diagnostics = DiagnosticsBuilder::default();
    let elements = Parser::parse_file(&simple_db, &mut diagnostics, virtual_file, code.as_str())
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
