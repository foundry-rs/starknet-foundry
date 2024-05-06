use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments, attributes::AttributeCollector, config_fn::ExtendWithConfig, MacroResult,
};
use cairo_lang_macro::{Diagnostics, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct IgnoreCollector;

impl AttributeInfo for IgnoreCollector {
    const ATTR_NAME: &'static str = "ignore";
    const ARGS_FORM: &'static str = "";
}

impl AttributeTypeData for IgnoreCollector {
    const CHEATCODE_NAME: &'static str = "set_config_ignore";
}

impl AttributeCollector for IgnoreCollector {
    fn args_into_body(_db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics> {
        args.assert_is_empty::<Self>()?;

        Ok("snforge_std::_config_types::IgnoreConfig {{ is_ignored: true }}".to_string())
    }
}

pub fn _ignore(args: TokenStream, item: TokenStream) -> MacroResult {
    IgnoreCollector::extend_with_config_cheatcodes(args, item)
}
