// use super::{AttributeCollector, AttributeInfo, AttributeTypeData};
// use crate::args::Arguments;
// use crate::asserts::assert_is_used_once;
// use crate::attributes::test_case::wrapper::TestCaseWrapperCollector;
// use crate::common::into_proc_macro_result;
// use crate::config_statement::extend_with_config_cheatcodes;
// use crate::parse::parse;
// use crate::utils::create_single_token;
// use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
// use cairo_lang_parser::utils::SimpleParserDatabase;
// use cairo_lang_syntax::node::TypedSyntaxNode;
// use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
// use cairo_lang_utils::Upcast;

// pub mod wrapper;

// pub struct TestCaseConfigCollector;

// impl AttributeInfo for TestCaseConfigCollector {
//     const ATTR_NAME: &'static str = "__test_case_config";
// }

// pub struct TestCaseCollector;

// impl AttributeInfo for TestCaseCollector {
//     const ATTR_NAME: &'static str = "test_case";
// }

// impl AttributeTypeData for TestCaseCollector {
//     const CHEATCODE_NAME: &'static str = "set_config_test_case";
// }

// impl AttributeCollector for TestCaseCollector {
//     fn args_into_config_expression(
//         _db: &SimpleParserDatabase,
//         args: Arguments,
//         _warns: &mut Vec<Diagnostic>,
//     ) -> Result<TokenStream, Diagnostics> {
//         let unnamed_args = args.unnamed_only::<TestCaseCollector>()?;
//         println!("length of unnamed_args: {}", unnamed_args.len());
//         for arg in unnamed_args.iter() {
//             println!("test_case arg: {:?}", arg);
//         }
//         Ok(quote!(unnamed_args))
//     }
// }

// #[must_use]
// pub fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
//     into_proc_macro_result(args, item, test_case_internal)
// }

// #[must_use]
// pub fn test_case_config(args: TokenStream, item: TokenStream) -> ProcMacroResult {
//     extend_with_config_cheatcodes::<TestCaseCollector>(args, item)
// }

// #[expect(clippy::ptr_arg)]
// fn test_case_internal(
//     args: &TokenStream,
//     item: &TokenStream,
//     _warns: &mut Vec<Diagnostic>,
// ) -> Result<TokenStream, Diagnostics> {
//     let (db, func) = parse::<TestCaseCollector>(item)?;
//     let db = db.upcast();

//     assert_is_used_once::<TestCaseCollector>(db, &func)?;

//     let attrs = func.attributes(db).as_syntax_node();
//     let attrs = SyntaxNodeWithDb::new(&attrs, db);

//     let body = func.body(db).as_syntax_node();
//     let body: SyntaxNodeWithDb<'_, SimpleParserDatabase> = SyntaxNodeWithDb::new(&body, db);

//     let declaration = func.declaration(db).as_syntax_node();
//     let declaration = SyntaxNodeWithDb::new(&declaration, db);

//     let test_case_config = create_single_token(TestCaseConfigCollector::ATTR_NAME);
//     let test_case_wrapper = create_single_token(TestCaseWrapperCollector::ATTR_NAME);

//     let args = args.clone();

//     Ok(quote!(
//         #[#test_case_config]
//         #[#test_case_wrapper #args]
//         #attrs
//         #declaration
//             #body
//     ))
// }

// src/attributes/param_test.rs

use std::ops::Deref;

use super::AttributeInfo;
use crate::attributes::ErrorExt;
use crate::common::into_proc_macro_result;
use crate::parse::parse;
use crate::utils::create_single_token;
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::TypedSyntaxNode;
use cairo_lang_syntax::node::helpers::QueryAttrs;
use cairo_lang_syntax::node::with_db::SyntaxNodeWithDb;
use cairo_lang_utils::Upcast;

pub mod wrapper;

pub struct ParamTestCollector;
impl AttributeInfo for ParamTestCollector {
    const ATTR_NAME: &'static str = "param_test";
}

pub struct TestCaseCollector;
impl AttributeInfo for TestCaseCollector {
    const ATTR_NAME: &'static str = "test_case";
}

#[must_use]
pub fn test_case(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |_, item, _| Ok(item.clone()))
}

#[must_use]
pub fn param_test(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    into_proc_macro_result(args, item, |_, item, _| {
        let (db_any, func) = parse::<ParamTestCollector>(&item)?;
        let db: &SimpleParserDatabase = db_any.upcast();

        let attrs = func.attributes(db);
        let has_any_case = attrs.find_attr(db, TestCaseCollector::ATTR_NAME).is_some();
        if !has_any_case {
            Err(wrapper::ParamTestWrapperCollector::error(
                "No #[test_case(...)] found. Add at least one.",
            ))?;
        }

        let has_wrapper = attrs
            .find_attr(db, wrapper::ParamTestWrapperCollector::ATTR_NAME)
            .is_some();

        let attrs = attrs.as_syntax_node();
        let attrs = SyntaxNodeWithDb::new(&attrs, db);
        let decl = func.declaration(db).as_syntax_node();
        let decl = SyntaxNodeWithDb::new(&decl, db);
        let body = func.body(db).as_syntax_node();
        let body = SyntaxNodeWithDb::new(&body, db);

        let wrapper_attr = create_single_token(wrapper::ParamTestWrapperCollector::ATTR_NAME);

        Ok(if has_wrapper {
            quote!( #attrs #decl #body )
        } else {
            quote!( #[#wrapper_attr] #attrs #decl #body )
        })
    })
}
