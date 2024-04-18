use crate::{
    args::Arguments,
    asserts::{assert_is_used_on_test, assert_is_used_once},
    attributes::{AttributeCollector, AttributeInfo},
    parse::{parse, parse_args},
    MacroResult,
};
use cairo_lang_macro::TokenStream;
use cairo_lang_syntax::node::Terminal;
use cairo_lang_utils::Upcast;
use indoc::formatdoc;

pub trait ConfigFn {
    fn get_config_fn_name(origin_fn_name: &str) -> String;
    fn create_config_fn(origin_fn_name: &str, body: &str) -> String;
    fn extend_with_config_fn(args: TokenStream, item: TokenStream) -> MacroResult;
}

impl<T> ConfigFn for T
where
    T: AttributeCollector + AttributeInfo,
{
    fn get_config_fn_name(origin_fn_name: &str) -> String {
        let attr_name = Self::ATTR_NAME;

        format!("snforge_internal_prefix_{origin_fn_name}_{attr_name}")
    }

    fn create_config_fn(origin_fn_name: &str, body: &str) -> String {
        let fn_name = Self::get_config_fn_name(origin_fn_name);
        let return_type = Self::RETURN_TYPE;

        format!("fn {fn_name}()->Option<snforge_std::_config_types::{return_type}>{{{body}}}")
    }

    fn extend_with_config_fn(args: TokenStream, item: TokenStream) -> MacroResult {
        let item = item.to_string();
        let (db, func) = parse::<Self>(&item)?;

        let db = db.upcast();

        assert_is_used_once::<Self>(db, &func)?;
        assert_is_used_on_test::<Self>(db, &func)?;

        let (args_db, args) = parse_args::<Self>(&args.to_string())?;

        let (args, empty_args_list_warn) = Arguments::new::<Self>(args_db.upcast(), args);

        let fn_body = Self::args_into_body(args_db.upcast(), args).map_err(|err| {
            if let Some(empty_args_list_warn) = empty_args_list_warn {
                err.warn(empty_args_list_warn)
            } else {
                err
            }
        })?;

        let return_type = Self::RETURN_TYPE;
        let config_fn = Self::create_config_fn(
            func.declaration(db).name(db).text(db).as_str(),
            &format!("Option::Some(snforge_std::_config_types::{return_type}{{{fn_body}}})",),
        );

        Ok(TokenStream::new(formatdoc!(
            "
                {config_fn}
                {item}
            "
        )))
    }
}
