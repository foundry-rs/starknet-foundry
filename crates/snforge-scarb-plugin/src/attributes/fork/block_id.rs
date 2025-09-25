use super::ForkCollector;
use crate::{
    attributes::ErrorExt,
    cairo_expression::CairoExpression,
    types::{Number, ParseFromExpr},
};
use cairo_lang_macro::{Diagnostic, TokenStream, quote};
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::{ast::Expr, helpers::GetIdentifier};

#[derive(Debug, Clone, Copy)]
pub enum BlockIdVariants {
    Hash,
    Number,
    Tag,
}

impl AsRef<str> for BlockIdVariants {
    fn as_ref(&self) -> &str {
        match self {
            Self::Hash => "block_hash",
            Self::Number => "block_number",
            Self::Tag => "block_tag",
        }
    }
}

#[derive(Debug, Clone)]
pub enum BlockId {
    Hash(Number),
    Number(Number),
    Tag,
}

impl CairoExpression for BlockId {
    fn as_cairo_expression(&self) -> TokenStream {
        match self {
            Self::Hash(hash) => {
                let block_hash = hash.as_cairo_expression();
                quote!(snforge_std::_internals::config_types::BlockId::BlockHash(#block_hash))
            }
            Self::Number(number) => {
                let block_number = number.as_cairo_expression();
                quote!(snforge_std::_internals::config_types::BlockId::BlockNumber(#block_number))
            }
            Self::Tag => quote!(snforge_std::_internals::config_types::BlockId::BlockTag),
        }
    }
}

impl ParseFromExpr<(BlockIdVariants, &Expr)> for BlockId {
    fn parse_from_expr<T: crate::attributes::AttributeInfo>(
        db: &SimpleParserDatabase,
        (variant, block_args): &(BlockIdVariants, &Expr),
        arg_name: &str,
    ) -> Result<Self, Diagnostic> {
        match variant {
            BlockIdVariants::Tag => {
                if let Expr::Path(path) = block_args {
                    let segments = path.segments(db).elements(db);

                    if segments.len() == 1 {
                        let segment = segments.last().unwrap();

                        // currently no other tags
                        if segment.identifier(db).as_str() == "latest" {
                            return Ok(Self::Tag);
                        }
                    }
                }
                Err(ForkCollector::error(format!(
                    "<{arg_name}> value incorrect, expected: latest",
                )))
            }
            BlockIdVariants::Hash => {
                let hash = Number::parse_from_expr::<ForkCollector>(
                    db,
                    block_args,
                    BlockIdVariants::Hash.as_ref(),
                )?;

                Ok(Self::Hash(hash))
            }
            BlockIdVariants::Number => {
                let number = Number::parse_from_expr::<ForkCollector>(
                    db,
                    block_args,
                    BlockIdVariants::Number.as_ref(),
                )?;

                Ok(Self::Number(number))
            }
        }
    }
}
