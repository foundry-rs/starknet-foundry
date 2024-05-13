use super::{AttributeInfo, AttributeTypeData};
use crate::{args::Arguments, attributes::AttributeCollector, validate};
use cairo_lang_macro::{Diagnostic, Diagnostics};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct FuzzerCollector;

impl AttributeInfo for FuzzerCollector {
    const ATTR_NAME: &'static str = "fuzzer";
    const ARGS_FORM: &'static str = "<runs>: `u64`, <seed>: `felt252`";
}

impl AttributeTypeData for FuzzerCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fuzzer";
}

impl AttributeCollector for FuzzerCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let seed = validate::maybe_number_value::<Self>(db, named_args, "seed")?;
        let runs = validate::maybe_number_value::<Self>(db, named_args, "runs")?;

        Ok(format!(
            "snforge_std::_config_types::FuzzerConfig {{ seed: {seed}, runs: {runs} }}"
        ))
    }
}
