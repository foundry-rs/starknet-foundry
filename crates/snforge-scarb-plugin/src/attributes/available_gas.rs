use super::{AttributeInfo, AttributeTypeData};
use crate::{args::Arguments, attributes::AttributeCollector, validate};
use cairo_lang_macro::Diagnostics;
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct AvailableGasCollector;

impl AttributeInfo for AvailableGasCollector {
    const ATTR_NAME: &'static str = "available_gas";
    const ARGS_FORM: &'static str = "<usize>";
}

impl AttributeTypeData for AvailableGasCollector {
    const CHEATCODE_NAME: &'static str = "set_config_available_gas";
}

impl AttributeCollector for AvailableGasCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
    ) -> Result<String, Diagnostics> {
        let [arg] = args.unnamed_only::<Self>()?.of_length::<1>()?;

        let gas = validate::number::<Self>(db, arg, "0")?;

        Ok(format!(
            "snforge_std::_config_types::AvailableGasConfig {{ gas: {gas} }}"
        ))
    }
}
