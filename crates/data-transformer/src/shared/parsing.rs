use cairo_lang_diagnostics::DiagnosticsBuilder;
use cairo_lang_filesystem::ids::{FileKind, FileLongId, VirtualFile};
use cairo_lang_parser::parser::Parser;
use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::Expr;
use cairo_lang_utils::Intern;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid Cairo expression found in input calldata \"{expr}\":\n{diagnostics}")]
    InvalidExpression { expr: String, diagnostics: String },
}

pub fn parse_expression(source: &str, db: &SimpleParserDatabase) -> Result<Expr, ParseError> {
    let file = FileLongId::Virtual(VirtualFile {
        parent: None,
        name: "parser_input".into(),
        content: source.to_string().into(),
        code_mappings: [].into(),
        kind: FileKind::Expr,
        original_item_removed: false,
    })
    .intern(db);

    let mut diagnostics = DiagnosticsBuilder::default();
    let expression = Parser::parse_file_expr(db, &mut diagnostics, file, source);
    let diagnostics = diagnostics.build();

    if diagnostics.check_error_free().is_err() {
        return Err(ParseError::InvalidExpression {
            expr: source.to_string(),
            diagnostics: diagnostics.format(db),
        });
    }

    Ok(expression)
}
