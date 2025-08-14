use crate::args::Arguments;
use cairo_lang_macro::{Diagnostic, Diagnostics, TokenStream};
use cairo_lang_parser::utils::SimpleParserDatabase;

pub mod available_gas;
pub mod disable_predeployed_contracts;
pub mod fork;
pub mod fuzzer;
pub mod ignore;
pub mod internal_config_statement;
pub mod should_panic;
pub mod test;
pub mod test_case;

pub trait AttributeInfo {
    const ATTR_NAME: &'static str;
}

pub trait AttributeTypeData {
    const CHEATCODE_NAME: &'static str;
}

pub trait AttributeCollector: AttributeInfo + AttributeTypeData {
    fn args_into_config_expression(
        db: &SimpleParserDatabase,
        args: Arguments,
        warns: &mut Vec<Diagnostic>,
    ) -> Result<TokenStream, Diagnostics>;
}

pub trait ErrorExt {
    fn error(message: impl ToString) -> Diagnostic;
    fn warn(message: impl ToString) -> Diagnostic;
}

impl<T> ErrorExt for T
where
    T: AttributeInfo,
{
    fn error(message: impl ToString) -> Diagnostic {
        let message = message.to_string();
        let attr_name = Self::ATTR_NAME;

        Diagnostic::error(format!("#[{attr_name}] {message}"))
    }

    fn warn(message: impl ToString) -> Diagnostic {
        let message = message.to_string();
        let attr_name = Self::ATTR_NAME;

        Diagnostic::warn(format!("#[{attr_name}] {message}"))
    }
}
