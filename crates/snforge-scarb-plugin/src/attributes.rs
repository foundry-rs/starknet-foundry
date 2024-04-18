use crate::args::Arguments;
use cairo_lang_macro::{Diagnostic, Diagnostics};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub mod available_gas;
pub mod fork;
pub mod fuzzer;
pub mod ignore;
pub mod should_panic;
pub mod test;

pub trait AttributeInfo: Sized {
    const ATTR_NAME: &'static str;
    const ARGS_FORM: &'static str;
}

pub trait AttributeReturnType: Sized {
    const RETURN_TYPE: &'static str;
}

pub trait AttributeCollector: AttributeInfo + AttributeReturnType {
    fn args_into_body(db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics>;
}

pub trait ErrorExt {
    fn error(message: impl ToString) -> Diagnostic;
}

impl<T> ErrorExt for T
where
    T: AttributeInfo,
{
    fn error(message: impl ToString) -> Diagnostic {
        let message = message.to_string();

        Diagnostic::error(format!("#[{}] {message}", Self::ATTR_NAME))
    }
}
