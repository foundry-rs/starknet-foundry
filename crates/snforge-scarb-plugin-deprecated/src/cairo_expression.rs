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

impl<T> CairoExpression for Vec<T>
where
    T: CairoExpression,
{
    fn as_cairo_expression(&self) -> String {
        let mut result = "array![".to_string();

        for e in self {
            result.push_str(&e.as_cairo_expression());

            result.push(',');
        }

        result.push(']');

        result
    }
}
