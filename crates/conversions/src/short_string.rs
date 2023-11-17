use crate::FromConv;
use cairo_felt::Felt252;
use cairo_lang_runner::short_string::as_cairo_short_string;

impl FromConv<Felt252> for String {
    fn from_(value: Felt252) -> String {
        as_cairo_short_string(&value).expect("Conversion to short string failed")
    }
}
