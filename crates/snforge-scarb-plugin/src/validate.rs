use crate::{
    args::named::NamedArgs,
    attributes::{AttributeInfo, ErrorExt},
};
use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup};
use url::Url;

pub fn maybe_number_value<T: AttributeInfo>(
    db: &dyn SyntaxGroup,
    named_args: &NamedArgs,
    arg_name: &str,
) -> Result<String, Diagnostic> {
    let arg = named_args.as_once_optional(arg_name)?;

    arg.map_or(Ok("Option::None".to_string()), |arg| {
        number::<T>(db, arg, arg_name).map(|num| format!("Option::Some({num})"))
    })
}

pub fn number<T: AttributeInfo>(
    db: &dyn SyntaxGroup,
    num: &Expr,
    arg: &str,
) -> Result<String, Diagnostic> {
    match num {
        Expr::Literal(literal) => {
            let num = literal
                .numeric_value(db)
                .ok_or_else(|| T::error(format!("<{arg}> got invalid number literal")))?
                .to_str_radix(16);

            Ok(format!("0x{num}"))
        }
        _ => Err(T::error(format!("<{arg}> should be number literal",))),
    }
}

pub fn url<T: AttributeInfo>(
    db: &dyn SyntaxGroup,
    url: &Expr,
    arg: &str,
) -> Result<String, Diagnostic> {
    let url = string::<T>(db, url, arg)?;

    match Url::parse(&url) {
        Ok(_) => Ok(url),
        Err(_) => Err(T::error(format!("<{arg}> is not a valid url"))),
    }
}

pub fn string<T: AttributeInfo>(
    db: &dyn SyntaxGroup,
    url: &Expr,
    arg: &str,
) -> Result<String, Diagnostic> {
    match url {
        Expr::String(string) => match string.string_value(db) {
            None => Err(T::error(format!("<{arg}> is not a valid string"))),
            Some(string) => Ok(string),
        },
        _ => Err(T::error(format!(
            "<{arg}> invalid type, should be: double quotted string"
        ))),
    }
}
