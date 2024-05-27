use cairo_lang_macro::Diagnostic;
use cairo_lang_syntax::node::ast::Expr;
use std::{collections::HashMap, ops::Deref};

pub struct UnnamedArgs<'a>(Vec<&'a Expr>);

impl<'a> Deref for UnnamedArgs<'a> {
    type Target = Vec<&'a Expr>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl UnnamedArgs<'_> {
    pub fn new(unnamed: &HashMap<usize, Expr>) -> UnnamedArgs<'_> {
        let mut args: Vec<_> = unnamed.iter().collect();

        args.sort_by(|(a, _), (b, _)| a.cmp(b));

        let args = args.into_iter().map(|(_, expr)| expr).collect();

        UnnamedArgs(args)
    }
}

impl<'a> UnnamedArgs<'a> {
    pub fn of_length<const T: usize>(&self) -> Result<&[&'a Expr; T], Diagnostic> {
        self.as_slice().try_into().map_err(|_| {
            Diagnostic::error(format!("expected {} arguments, got: {}", T, self.len()))
        })
    }
}
