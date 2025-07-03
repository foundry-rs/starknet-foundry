use cairo_lang_parser::utils::SimpleParserDatabase;
use cairo_lang_syntax::node::ast::{
    ExprPath, GenericArg, GenericArgValue, PathSegment, PathSegmentWithGenericArgs,
};
use cairo_lang_syntax::node::{Token, TypedSyntaxNode};

#[derive(Debug, thiserror::Error)]
pub enum PathSplitError {
    #[error("Invalid generic arguments")]
    InvalidGenericArgs,
    #[error("Expected exactly one generic argument")]
    MoreThanOneGenericArg,
    #[error("Path segment missing")]
    PathSegmentMissing,
}

pub enum SplitResult {
    Simple {
        splits: Vec<String>,
    },
    WithGenericArgs {
        splits: Vec<String>,
        generic_args: String,
    },
}

/// Splits a path into its segments, and extracts generic arguments if present.
///
/// In the case of Cairo-like language constructs such as arrays or spans,
/// we assume that if there are generic arguments (e.g., `Span<T>`), they
/// appear at the end of the path. Therefore, by the time we encounter a
/// segment with generic arguments, all preceding segments have already
/// been collected.
///
/// For example, in a path like `core::array::Array<felt252>`, this function will:
/// - Collect "core", "array", and "Array" into `splits`
/// - Extract the generic argument `felt252` from `Array<felt252>`
pub fn split(path: &ExprPath, db: &SimpleParserDatabase) -> Result<SplitResult, PathSplitError> {
    let mut splits = Vec::new();
    let elements = path.segments(db).elements(db);
    for (i, p) in elements.iter().enumerate() {
        match p {
            PathSegment::Simple(segment) => {
                splits.push(segment.ident(db).token(db).text(db).to_string());
            }
            PathSegment::WithGenericArgs(segment) => {
                splits.push(segment.ident(db).token(db).text(db).to_string());
                let generic_args = extract_generic_args(segment, db)?;

                let is_last = i == elements.len() - 1;
                return if is_last {
                    Ok(SplitResult::WithGenericArgs {
                        splits,
                        generic_args,
                    })
                } else {
                    Err(PathSplitError::InvalidGenericArgs)
                };
            }
            PathSegment::Missing(_segment) => Err(PathSplitError::PathSegmentMissing)?,
        }
    }

    Ok(SplitResult::Simple { splits })
}

fn extract_generic_args(
    segment: &PathSegmentWithGenericArgs,
    db: &SimpleParserDatabase,
) -> Result<String, PathSplitError> {
    let generic_args = segment
        .generic_args(db)
        .generic_args(db)
        .elements(db)
        .into_iter()
        .map(|arg| match arg {
            GenericArg::Named(_) => Err(PathSplitError::InvalidGenericArgs),
            GenericArg::Unnamed(arg) => match arg.value(db) {
                GenericArgValue::Expr(expr) => Ok(expr.as_syntax_node().get_text(db)),
                GenericArgValue::Underscore(_) => Err(PathSplitError::InvalidGenericArgs),
            },
        })
        .collect::<Result<Vec<_>, PathSplitError>>()?;

    let [generic_arg] = generic_args.as_slice() else {
        return Err(PathSplitError::MoreThanOneGenericArg);
    };

    Ok(generic_arg.clone())
}
