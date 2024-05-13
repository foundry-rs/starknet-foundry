use super::{AttributeInfo, AttributeTypeData};
use crate::{
    args::Arguments,
    attributes::{AttributeCollector, ErrorExt},
    utils::branch,
    validate,
};
use cairo_lang_macro::{Diagnostic, Diagnostics};
use cairo_lang_syntax::node::{ast::Expr, db::SyntaxGroup, helpers::GetIdentifier};
pub struct ForkCollector;

impl AttributeInfo for ForkCollector {
    const ATTR_NAME: &'static str = "fork";
    const ARGS_FORM: &'static str = "<url>: `ByteArray`, (<block_hash>: `felt252` | <block_number>: `felt252` | <block_tag>: latest)";
}

impl AttributeTypeData for ForkCollector {
    const CHEATCODE_NAME: &'static str = "set_config_fork";
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

fn inline_args(db: &dyn SyntaxGroup, args: &Arguments) -> Result<String, Diagnostic> {
    let named_args = args.named_only::<ForkCollector>()?;

    let (block_id, block_args) =
        named_args.one_of_once(&[BlockId::Hash, BlockId::Number, BlockId::Tag])?;

    let url = named_args.as_once("url")?;
    let url = validate::url::<ForkCollector>(db, url)?;

    let block_id_value = validate_block_id(db, block_id, block_args)?;

    let block_id_value = match block_id {
        BlockId::Hash => format!("BlockHash({block_id_value})"),
        BlockId::Number => format!("BlockNumber({block_id_value})"),
        BlockId::Tag => "BlockTag".to_string(),
    };

    Ok(format!("snforge_std::_config_types::ForkConfig::Inline(snforge_std::_config_types::InlineForkConfig {{ url: {url}, block: {block_id_value} }})"))
}

fn from_file_args(db: &dyn SyntaxGroup, args: &Arguments) -> Result<String, Diagnostic> {
    let [arg] = args.unnamed_only::<ForkCollector>()?.of_length::<1>()?;

    let name = validate::string::<ForkCollector>(db, arg)?;

    Ok(format!(
        r#"snforge_std::_config_types::ForkConfig::Named("{name}")"#
    ))
}

impl AttributeCollector for ForkCollector {
    fn args_into_config_expression(
        db: &dyn SyntaxGroup,
        args: Arguments,
    ) -> Result<String, Diagnostics> {
        branch(|| inline_args(db, &args), || from_file_args(db, &args))
    }
}

fn validate_block_id(
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
            Err(ForkCollector::error(format!(
                "<{}> value incorrect, expected: latest",
                BlockId::Tag.as_str(),
            )))
        }
        BlockId::Hash | BlockId::Number => {
            crate::validate::number::<ForkCollector>(db, block_args, block_id.as_str())
        }
    }
}
