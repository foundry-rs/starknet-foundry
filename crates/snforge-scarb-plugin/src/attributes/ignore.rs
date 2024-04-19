use super::{AttributeInfo, AttributeReturnType};
use crate::{
    args::Arguments, asserts::assert_is_empty, attributes::AttributeCollector, config_fn::ConfigFn,
    MacroResult,
};
use cairo_lang_macro::{Diagnostics, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct IgnoreCollector;

impl AttributeInfo for IgnoreCollector {
    const ATTR_NAME: &'static str = "ignore";
    const ARGS_FORM: &'static str = "";
}

impl AttributeReturnType for IgnoreCollector {
    const RETURN_TYPE: &'static str = "IgnoreConfig";
    const EXECUTABLE_NAME: &'static str = "__snforge_ignore__";
}

impl AttributeCollector for IgnoreCollector {
    fn args_into_body(_db: &dyn SyntaxGroup, _args: Arguments) -> Result<String, Diagnostics> {
        Ok("is_ignored: true".to_string())
    }
}

pub fn _ignore(args: TokenStream, item: TokenStream) -> MacroResult {
    assert_is_empty::<IgnoreCollector>(&args.to_string())?;

    IgnoreCollector::extend_with_config_fn(args, item)
}
