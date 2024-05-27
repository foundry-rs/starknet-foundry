use self::block_id::{BlockId, BlockIdVariants};
use crate::{
    args::Arguments,
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::ParseFromExpr,
    utils::branch,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;
use indoc::formatdoc;
use url::Url;

mod block_id;

pub struct ForkCollector;

impl AttributeInfo for ForkCollector {
    const ATTR_NAME: &'static str = "fork";
    const ARGS_FORM: &'static str = "<url>: `String`, (<block_hash>: `felt252` | <block_number>: `felt252` | <block_tag>: latest)";
}

impl AttributeTypeData for ForkCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fork";
}

impl AttributeCollector for ForkCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<String, Diagnostics> {
        branch(inline_args(db, &args), || from_file_args(db, &args))
    }
}

fn inline_args(db: &dyn SyntaxGroup, args: &Arguments) -> Result<String, Diagnostic> {
    let named_args = args.named_only::<ForkCollector>()?;

    let block_id = named_args.one_of_once(&[
        BlockIdVariants::Hash,
        BlockIdVariants::Number,
        BlockIdVariants::Tag,
    ])?;
    let url = named_args.as_once("url")?;

    let block_id = BlockId::parse_from_expr::<ForkCollector>(db, &block_id, block_id.0.as_ref())?;
    let url = Url::parse_from_expr::<ForkCollector>(db, url, "url")?;

    let block_id = block_id.as_cairo_expression();

    Ok(formatdoc!(
        "
            snforge_std::_config_types::ForkConfig::Inline(
                snforge_std::_config_types::InlineForkConfig {{
                    url: {url},
                    block: {block_id}
                }}
            )
        "
    ))
}

fn from_file_args(db: &dyn SyntaxGroup, args: &Arguments) -> Result<String, Diagnostic> {
    let &[arg] = args.unnamed_only::<ForkCollector>()?.of_length::<1>()?;

    let name = String::parse_from_expr::<ForkCollector>(db, arg, "0")?;

    Ok(format!(
        r#"snforge_std::_config_types::ForkConfig::Named("{name}")"#
    ))
}

pub fn fork(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<ForkCollector>(args, item)
}
