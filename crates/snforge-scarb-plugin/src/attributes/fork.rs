use self::block_id::{BlockId, BlockIdVariants};
use crate::{
    args::Arguments,
    attributes::{AttributeCollector, AttributeInfo, AttributeTypeData},
    branch,
    cairo_expression::CairoExpression,
    config_statement::extend_with_config_cheatcodes,
    types::ParseFromExpr,
};
use cairo_lang_macro::{Diagnostic, Diagnostics, ProcMacroResult, Severity, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use url::Url;

mod block_id;

pub struct ForkCollector;

impl AttributeInfo for ForkCollector {
    const ATTR_NAME: &'static str = "fork";
}

impl AttributeTypeData for ForkCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fork";
}

impl AttributeCollector for ForkCollector {
    fn args_into_config_expression(
        db: &SimpleParserDatabase,
        args: Arguments,
        _warns: &mut Vec<Diagnostic>,
    ) -> Result<TokenStream, Diagnostics> {
        let expr = branch!(
            inline_args(db, &args),
            overridden_args(db, &args),
            from_file_args(db, &args)
        )?;

        Ok(expr)
    }
}

fn inline_args(db: &SimpleParserDatabase, args: &Arguments) -> Result<TokenStream, Diagnostic> {
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
    let url = url.as_cairo_expression();

    Ok(quote!(
        snforge_std::_internals::config_types::ForkConfig::Inline(
            snforge_std::_internals::config_types::InlineForkConfig {
                url: #url,
                block: #block_id
            }
        )
    ))
}

fn from_file_args(db: &SimpleParserDatabase, args: &Arguments) -> Result<TokenStream, Diagnostic> {
    let &[arg] = args
        .unnamed_only::<ForkCollector>()?
        .of_length::<1, ForkCollector>()?;

    let name = String::parse_from_expr::<ForkCollector>(db, arg.1, arg.0.to_string().as_str())?;

    let name = name.as_cairo_expression();

    Ok(quote!(snforge_std::_internals::config_types::ForkConfig::Named(#name)))
}

fn overridden_args(db: &SimpleParserDatabase, args: &Arguments) -> Result<TokenStream, Diagnostic> {
    let &[arg] = args.unnamed().of_length::<1, ForkCollector>()?;

    let named_args = args.named();
    let block_id = named_args.one_of_once(&[
        BlockIdVariants::Hash,
        BlockIdVariants::Number,
        BlockIdVariants::Tag,
    ])?;

    let block_id = BlockId::parse_from_expr::<ForkCollector>(db, &block_id, block_id.0.as_ref())?;
    let name = String::parse_from_expr::<ForkCollector>(db, arg.1, arg.0.to_string().as_str())?;

    let block_id = block_id.as_cairo_expression();
    let name = name.as_cairo_expression();

    Ok(quote!(
        snforge_std::_internals::config_types::ForkConfig::Overridden(
            snforge_std::_internals::config_types::OverriddenForkConfig {
                block: #block_id,
                name: #name
            }
        )
    ))
}

#[must_use]
pub fn fork(args: TokenStream, item: TokenStream) -> ProcMacroResult {
    extend_with_config_cheatcodes::<ForkCollector>(args, item)
}
