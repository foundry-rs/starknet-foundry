use crate::attributes::{AttributeInfo, ErrorExt};
use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::ast::Expr;
use std::{collections::HashMap, ops::Deref};

pub struct UnnamedArgs<'a>(Vec<(usize, &'a Expr)>);

impl<'a> Deref for UnnamedArgs<'a> {
    type Target = Vec<(usize, &'a Expr)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnnamedArgs<'_> {
    pub fn new(unnamed: &HashMap<usize, Expr>) -> UnnamedArgs<'_> {
        let mut args: Vec<_> = unnamed.iter().collect();

        args.sort_by(|(a, _), (b, _)| a.cmp(b));

        let args = args.into_iter().map(|(&pos, expr)| (pos, expr)).collect();

        UnnamedArgs(args)
    }
}

impl<'a> UnnamedArgs<'a> {
    pub fn of_length<const N: usize, T: AttributeInfo>(
        &self,
    ) -> Result<&[(usize, &'a Expr); N], Diagnostic> {
        self.as_slice()
            .try_into()
            .map_err(|_| T::error(format!("expected {} arguments, got: {}", N, self.len())))
    }
}
