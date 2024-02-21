use anyhow::Error;
use console::style;

pub fn print_as_warning(error: &Error) {
    let warning_tag = style("WARNING").color256(11);
    println!("[{warning_tag}] {error}");
}
