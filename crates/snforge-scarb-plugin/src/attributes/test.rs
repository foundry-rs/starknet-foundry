use crate::{
    asserts::{assert_is_empty, assert_is_used_once},
    attributes::AttributeInfo,
    parse::parse,
    MacroResult,
};
use cairo_lang_macro::TokenStream;
use cairo_lang_utils::Upcast;
use indoc::formatdoc;

pub struct TestCollector;

impl AttributeInfo for TestCollector {
    const ATTR_NAME: &'static str = "test";
    const ARGS_FORM: &'static str = "";
}

pub const SNFORGE_EXECUTABLE_TEST: &str = "__snforge_executable_test__";

#[allow(clippy::needless_pass_by_value)]
pub fn _test(args: TokenStream, item: TokenStream) -> MacroResult {
    let code = item.to_string();
    let (simple_db, func) = parse::<TestCollector>(&code)?;
    let db = simple_db.upcast();

    assert_is_used_once::<TestCollector>(db, &func)?;
    assert_is_empty::<TestCollector>(&args.to_string())?;

    Ok(TokenStream::new(formatdoc!(
        r#"
            #[executable("{SNFORGE_EXECUTABLE_TEST}")]
            {code}
        "#
    )))
}
