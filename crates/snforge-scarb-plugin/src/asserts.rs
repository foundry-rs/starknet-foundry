use crate::attributes::{AttributeInfo, ErrorExt};
use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup, helpers::QueryAttrs};

pub fn assert_is_used_once<T: AttributeInfo>(
    db: &dyn SyntaxGroup,
    func: &FunctionWithBody,
) -> Result<(), Diagnostic> {
    if func.attributes(db).has_attr(db, T::ATTR_NAME) {
        Err(T::error("can only be used once per item"))
    } else {
        Ok(())
    }
}
