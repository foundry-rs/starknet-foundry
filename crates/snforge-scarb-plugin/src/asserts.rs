use crate::{
    args::Arguments,
    attributes::{AttributeInfo, ErrorExt},
    parse::parse_args,
};
use cairo_lang_macro::{Diagnostic, Diagnostics};
use cairo_lang_syntax::node::{ast::FunctionWithBody, db::SyntaxGroup, helpers::QueryAttrs};
use cairo_lang_utils::Upcast;

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

pub fn assert_is_empty<T: AttributeInfo>(args: &str) -> Result<(), Diagnostics> {
    let (db, args) = parse_args::<T>(args)?;

    let (args, _warn) = Arguments::new::<T>(db.upcast(), args);

    if args.is_empty() {
        Ok(())
    } else {
        Err(T::error("does not accept any arguments"))?
    }
}
