use super::{internal_config_statement::InternalConfigStatementCollector, AttributeInfo};
use cairo_lang_macro::{ProcMacroResult, TokenStream};
use indoc::formatdoc;

struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
    const ARGS_FORM: &'static str = "";
}

pub fn test(item: &TokenStream) -> ProcMacroResult {
    let func = item.to_string();

    let config = InternalConfigStatementCollector::ATTR_NAME;

    let result = formatdoc!(
        "
            #[snforge_internal_test_executable]
            #[{config}]
            {func}
        "
    );

    ProcMacroResult::new(TokenStream::new(result))
}
