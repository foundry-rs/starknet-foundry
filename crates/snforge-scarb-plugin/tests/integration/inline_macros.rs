use cairo_lang_macro::TokenStream;
use snforge_scarb_plugin::create_single_token;

mod declare;

fn macro_args(path: &str) -> TokenStream {
    TokenStream::new(vec![create_single_token(format!("({path})"))])
}
