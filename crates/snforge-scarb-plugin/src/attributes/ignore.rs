use super::{AttributeInfo, AttributeTypeData};
use crate::{args::Arguments, attributes::AttributeCollector};
use cairo_lang_macro::Diagnostics;
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
    fn args_into_config_expression(_db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics> {
        args.assert_is_empty::<Self>()?;

        Ok("snforge_std::_config_types::IgnoreConfig {{ is_ignored: true }}".to_string())
    }
}
