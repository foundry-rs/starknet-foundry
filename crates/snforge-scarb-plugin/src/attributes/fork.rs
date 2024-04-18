use super::{AttributeInfo, AttributeReturnType};
use crate::{args::Arguments, attributes::AttributeCollector, config_fn::ConfigFn, MacroResult};
use cairo_lang_macro::{Diagnostics, TokenStream};
use cairo_lang_syntax::node::db::SyntaxGroup;

pub struct ForkCollector;

impl AttributeInfo for ForkCollector {
    const ATTR_NAME: &'static str = "fork";
    const ARGS_FORM: &'static str = "<url>: `ByteArray`, (<block_hash>: `felt252` | <block_number>: `felt252` | <block_tag>: latest)";
}

impl AttributeReturnType for ForkCollector {
    const RETURN_TYPE: &'static str = "ForkConfig";
}

#[derive(Debug, Clone, Copy)]
enum BlockId {
    Hash,
    Number,
    Tag,
}

impl From<BlockId> for &str {
    fn from(value: BlockId) -> Self {
        match value {
            BlockId::Hash => "block_hash",
            BlockId::Number => "block_number",
            BlockId::Tag => "block_tag",
        }
    }
}
impl BlockId {
    fn as_str(self) -> &'static str {
        self.into()
    }
}

impl AttributeCollector for ForkCollector {
    fn args_into_body(db: &dyn SyntaxGroup, args: Arguments) -> Result<String, Diagnostics> {
        let named_args = args.named_only::<Self>()?;

        let (block_id, block_args) =
            named_args.one_of_once(&[BlockId::Hash, BlockId::Number, BlockId::Tag])?;

        let url = named_args.as_once("url")?;
        let url = validate::url::<Self>(db, url)?;

        let block_id_value = validate::block_id::<Self>(db, block_id, block_args)?;

        let block_id_value = match block_id {
            BlockId::Hash => format!("BlockHash({block_id_value})"),
            BlockId::Number => format!("BlockNumber({block_id_value})"),
            BlockId::Tag => "BlockTag".to_string(),
        };

        Ok(format!("url: {url}, block: {block_id_value}"))
    }
}

pub fn _fork(args: TokenStream, item: TokenStream) -> MacroResult {
    ForkCollector::extend_with_config_fn(args, item)
}

mod validate {
    use super::BlockId;
    use crate::attributes::{fuzzer, AttributeInfo, ErrorExt};
    use cairo_lang_macro::Diagnostic;
    use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup, helpers::GetIdentifier};
    use url::Url;

    pub fn url<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        url: &Expr,
    ) -> Result<String, Diagnostic> {
        match url {
            Expr::String(string) => match string.string_value(db) {
                None => Err(T::error("<url> is not a valid string")),
                Some(url) => match Url::parse(&url) {
                    Ok(_) => Ok(url),
                    Err(_) => Err(T::error("<url> is not a valid url")),
                },
            },
            _ => Err(T::error(
                "<url> invalid type, should be: double quotted string",
            )),
        }
    }

    pub fn block_id<T: AttributeInfo>(
        db: &dyn SyntaxGroup,
        block_id: BlockId,
        block_args: &Expr,
    ) -> Result<String, Diagnostic> {
        match block_id {
            BlockId::Tag => {
                if let Expr::Path(path) = block_args {
                    let segments = path.elements(db);

                    if segments.len() == 1 {
                        let segment = segments.last().unwrap();

                        // currently no other tags
                        if segment.identifier(db).as_str() == "latest" {
                            return Ok(String::new());
                        }
                    }
                }
                Err(T::error(format!(
                    "<{}> value incorrect, expected: latest",
                    BlockId::Tag.as_str(),
                )))
            }
            BlockId::Hash | BlockId::Number => {
                fuzzer::validate::number::<T>(db, block_args, block_id.as_str())
            }
        }
    }
}
