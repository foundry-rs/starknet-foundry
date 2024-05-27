pub trait CairoExpression {
    fn as_cairo_expression(&self) -> String;
}

impl<T> CairoExpression for Option<T>
where
    T: CairoExpression,
{
    fn as_cairo_expression(&self) -> String {
        match self {
            Some(v) => format!("Option::Some({})", v.as_cairo_expression()),
            None => "Option::None".to_string(),
        }
    }
}
