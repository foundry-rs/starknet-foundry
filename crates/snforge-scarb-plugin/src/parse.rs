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
    let virtual_file = db.intern_file(FileLongId::Virtual(VirtualFile {
        parent: None,
        name: "test_function".into(),
        content: code.clone(),
        code_mappings: Default::default(),
        kind: FileKind::Module,
    }));
    let mut diagnostics = DiagnosticsBuilder::default();
    let elements = Parser::parse_file(&simple_db, &mut diagnostics, virtual_file, code.as_str())
        .items(db)
        .elements(db);

    if let Some(ModuleItem::FreeFunction(func)) = elements.into_iter().next() {
        Ok((simple_db, func))
    } else {
        Err(T::error("can be used only on a function"))
    }
}

struct InternalCollector;

impl AttributeInfo for InternalCollector {
    const ATTR_NAME: &'static str = "__INTERNAL_ATTR__";
    const ARGS_FORM: &'static str = "";
}

pub fn parse_args<T: AttributeInfo>(
    args: &str,
) -> Result<(SimpleParserDatabase, OptionArgListParenthesized), Diagnostic> {
    let (simple_db, func) = parse::<InternalCollector>(&formatdoc!(
        "
            #[{}{args}]
            fn __INTERNAL_FN__(){{}}
        ",
        InternalCollector::ATTR_NAME
    ))?;
    let db = simple_db.upcast();

    let args = func
        .attributes(db)
        .find_attr(db, InternalCollector::ATTR_NAME)
        .unwrap()
        .arguments(db);

    Ok((simple_db, args))
}
